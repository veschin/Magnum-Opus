use bevy::prelude::*;
use std::collections::HashSet;

use crate::components::*;
use crate::events::*;
use crate::resources::*;

const MAX_PATH_LENGTH: usize = 32;
const MINION_RANGE: i32 = 5;
const MINION_RATE: f32 = 0.5;

// ── Path creation system ──────────────────────────────────────────────────────

pub fn transport_placement_system(
    mut commands: Commands,
    mut transport_cmds: ResMut<TransportCommands>,
    mut path_occupancy: ResMut<PathOccupancy>,
    mut last_result: ResMut<LastDrawPathResult>,
    grid: Res<Grid>,
    transport_tier: Res<TransportTierState>,
    senders: Query<(Entity, &TransportSender)>,
    receivers: Query<Entity, With<TransportReceiver>>,
    mut ev_connected: MessageWriter<PathConnected>,
) {
    let cmds: Vec<DrawPathCmd> = transport_cmds.draw_path.drain(..).collect();
    for cmd in cmds {
        let sender_opt = senders.iter().find(|(e, _)| *e == cmd.source_group);
        if sender_opt.is_none() {
            last_result.result = Some(DrawPathResult::RejectedNoSender);
            continue;
        }
        if !receivers.iter().any(|e| e == cmd.target_group) {
            last_result.result = Some(DrawPathResult::RejectedNoReceiver);
            continue;
        }
        if cmd.waypoints.len() > MAX_PATH_LENGTH {
            last_result.result = Some(DrawPathResult::RejectedTooLong);
            continue;
        }
        let mut reject_reason: Option<DrawPathResult> = None;
        for &(wx, wy) in &cmd.waypoints {
            let terrain = grid.terrain_at(wx, wy);
            if terrain == TerrainType::LavaSource {
                reject_reason = Some(DrawPathResult::RejectedImpassable);
                break;
            }
            if path_occupancy.tiles.contains_key(&(wx, wy)) {
                reject_reason = Some(DrawPathResult::RejectedOccupied);
                break;
            }
        }
        if let Some(reason) = reject_reason {
            last_result.result = Some(reason);
            continue;
        }
        // Derive resource_filter from the source group's TransportSender so that
        // movement system can correctly match receiver demand by resource type.
        let resource_filter = sender_opt.and_then(|(_, s)| s.resource);
        let tier = transport_tier.transport_tier as u8;
        let stats = if cmd.is_pipe { TierStats::for_pipe(tier) } else { TierStats::for_path(tier) };
        let kind = if cmd.is_pipe { TransportKind::Pipe } else { TransportKind::RunePath };
        let path_entity = commands.spawn(TransportPath {
            kind,
            source_group: cmd.source_group,
            target_group: cmd.target_group,
            resource_filter,
            tier,
            capacity: stats.capacity,
            speed: stats.speed,
            connected: true,
            segments: cmd.waypoints.clone(),
        }).id();
        commands.spawn(PathConnection {
            source_group: cmd.source_group,
            target_group: cmd.target_group,
            path_entity,
        });
        for (idx, &(wx, wy)) in cmd.waypoints.iter().enumerate() {
            commands.spawn(PathSegmentTile {
                path_entity,
                tile_pos: (wx, wy),
                segment_index: idx,
            });
            path_occupancy.tiles.insert((wx, wy), path_entity);
        }
        ev_connected.write(PathConnected {
            path_entity,
            source_group: cmd.source_group,
            target_group: cmd.target_group,
        });
        last_result.result = Some(DrawPathResult::Ok);
    }
}

// ── Tier upgrade system ───────────────────────────────────────────────────────

pub fn transport_tier_upgrade_system(
    mut ev_tier: MessageReader<TierUnlocked>,
    mut transport_tier: ResMut<TransportTierState>,
    mut paths: Query<&mut TransportPath>,
) {
    let events: Vec<TierUnlocked> = ev_tier.read().cloned().collect();
    for ev in events {
        transport_tier.transport_tier = ev.tier as u32;
        for mut path in paths.iter_mut() {
            path.tier = ev.tier;
            let stats = match path.kind {
                TransportKind::RunePath => TierStats::for_path(ev.tier),
                TransportKind::Pipe => TierStats::for_pipe(ev.tier),
            };
            path.capacity = stats.capacity;
            path.speed = stats.speed;
        }
    }
}

// ── Transport movement system ─────────────────────────────────────────────────

pub fn transport_movement_system(
    mut commands: Commands,
    paths: Query<(Entity, &TransportPath)>,
    mut manifolds: Query<&mut Manifold, With<Group>>,
    senders_q: Query<(Entity, &TransportSender)>,
    receivers_q: Query<(Entity, &TransportReceiver)>,
    group_positions: Query<(Entity, &GroupPosition)>,
    mut cargos: Query<(Entity, &mut Cargo)>,
) {
    // Step 1: Advance and deliver existing cargo
    let mut to_despawn: Vec<Entity> = Vec::new();
    for (cargo_entity, mut cargo) in cargos.iter_mut() {
        if let Ok((_, path)) = paths.get(cargo.path_entity) {
            if !path.connected {
                to_despawn.push(cargo_entity);
                continue;
            }
            let path_len = path.segments.len() as f32;
            cargo.position_on_path += path.speed;
            if cargo.position_on_path >= path_len {
                if let Ok(mut manifold) = manifolds.get_mut(path.target_group) {
                    *manifold.resources.entry(cargo.resource).or_default() += cargo.amount;
                }
                to_despawn.push(cargo_entity);
            }
        } else {
            to_despawn.push(cargo_entity);
        }
    }
    for e in to_despawn { commands.entity(e).despawn(); }

    // Step 2: Launch new cargo
    for (path_entity, path) in paths.iter() {
        if !path.connected { continue; }
        let receiver_demand = receivers_q.iter()
            .find(|(e, r)| *e == path.target_group && r.resource == path.resource_filter)
            .map(|(_, r)| r.demand)
            .unwrap_or(path.capacity);
        let launch_cap = receiver_demand.min(path.capacity);
        if launch_cap == 0 { continue; }
        if let Ok(mut manifold) = manifolds.get_mut(path.source_group) {
            let resource = match path.resource_filter {
                Some(r) => r,
                None => {
                    let kind = path.kind;
                    let found = manifold.resources.iter()
                        .filter(|(r, amt)| **amt > 0.0 && match kind {
                            TransportKind::RunePath => r.class() == ResourceClass::Solid,
                            TransportKind::Pipe => r.class() == ResourceClass::Liquid,
                        })
                        .map(|(r, _)| *r).next();
                    match found { Some(r) => r, None => continue }
                }
            };
            let available = manifold.resources.get(&resource).copied().unwrap_or(0.0);
            if available <= 0.0 { continue; }
            let to_launch = (launch_cap as f32).min(available);
            *manifold.resources.entry(resource).or_default() -= to_launch;
            commands.spawn(Cargo {
                path_entity,
                resource,
                amount: to_launch,
                position_on_path: path.speed,
            });
        }
    }

    // Step 3: Minion carry fallback
    let connected_pairs: HashSet<(Entity, Entity)> = paths.iter()
        .filter(|(_, p)| p.connected)
        .map(|(_, p)| (p.source_group, p.target_group))
        .collect();
    let positions: Vec<(Entity, i32, i32)> = group_positions.iter()
        .map(|(e, p)| (e, p.x, p.y)).collect();

    for (src_entity, sender) in senders_q.iter() {
        let resource = match sender.resource { Some(r) => r, None => continue };
        if resource.class() == ResourceClass::Liquid { continue; }
        let (sx, sy) = match positions.iter().find(|(e, _, _)| *e == src_entity) {
            Some((_, x, y)) => (*x, *y), None => continue,
        };
        let mut nearest: Option<(Entity, i32)> = None;
        for (dst_entity, receiver) in receivers_q.iter() {
            if dst_entity == src_entity { continue; }
            if receiver.resource != Some(resource) { continue; }
            if receiver.demand == 0 { continue; }
            if connected_pairs.contains(&(src_entity, dst_entity)) { continue; }
            let (dx, dy) = match positions.iter().find(|(e, _, _)| *e == dst_entity) {
                Some((_, x, y)) => (*x, *y), None => continue,
            };
            let dist = (sx - dx).abs() + (sy - dy).abs();
            if dist > MINION_RANGE { continue; }
            match nearest {
                None => nearest = Some((dst_entity, dist)),
                Some((_, best)) if dist < best => nearest = Some((dst_entity, dist)),
                _ => {}
            }
        }
        if let Some((dst_entity, _)) = nearest {
            let available = manifolds.get(src_entity)
                .map(|m| m.resources.get(&resource).copied().unwrap_or(0.0))
                .unwrap_or(0.0);
            if available <= 0.0 { continue; }
            let to_transfer = MINION_RATE.min(available);
            if let Ok(mut src_m) = manifolds.get_mut(src_entity) {
                *src_m.resources.entry(resource).or_default() -= to_transfer;
            }
            if let Ok(mut dst_m) = manifolds.get_mut(dst_entity) {
                *dst_m.resources.entry(resource).or_default() += to_transfer;
            }
        }
    }
}

// ── Segment destruction system ────────────────────────────────────────────────

pub fn transport_destroy_system(
    mut commands: Commands,
    mut transport_cmds: ResMut<TransportCommands>,
    mut path_occupancy: ResMut<PathOccupancy>,
    mut paths: Query<&mut TransportPath>,
    segments: Query<(Entity, &PathSegmentTile)>,
    cargos: Query<(Entity, &Cargo)>,
    connections: Query<(Entity, &PathConnection)>,
    mut senders_q: Query<&mut TransportSender>,
    mut receivers_q: Query<&mut TransportReceiver>,
    mut ev_disconnected: MessageWriter<PathDisconnected>,
) {
    let tile_destroys: Vec<(i32, i32)> = transport_cmds.destroy_segment.drain(..).collect();
    let path_destroys: Vec<Entity> = transport_cmds.destroy_path.drain(..).collect();

    for tile_pos in tile_destroys {
        for (seg_entity, seg) in segments.iter() {
            if seg.tile_pos != tile_pos { continue; }
            let path_entity = seg.path_entity;
            commands.entity(seg_entity).despawn();
            path_occupancy.tiles.remove(&tile_pos);
            if let Ok(mut path) = paths.get_mut(path_entity) {
                path.connected = false;
                let src = path.source_group;
                let dst = path.target_group;
                if let Ok(mut s) = senders_q.get_mut(src) { s.disconnected = true; }
                if let Ok(mut r) = receivers_q.get_mut(dst) { r.disconnected = true; }
                ev_disconnected.write(PathDisconnected { path_entity, source_group: src, target_group: dst });
            }
            for (cargo_entity, cargo) in cargos.iter() {
                if cargo.path_entity == path_entity { commands.entity(cargo_entity).despawn(); }
            }
        }
    }

    for path_entity in path_destroys {
        if let Ok(path) = paths.get(path_entity) {
            let src = path.source_group;
            let dst = path.target_group;
            ev_disconnected.write(PathDisconnected { path_entity, source_group: src, target_group: dst });
            for (seg_entity, seg) in segments.iter() {
                if seg.path_entity == path_entity {
                    path_occupancy.tiles.remove(&seg.tile_pos);
                    commands.entity(seg_entity).despawn();
                }
            }
            for (cargo_entity, cargo) in cargos.iter() {
                if cargo.path_entity == path_entity { commands.entity(cargo_entity).despawn(); }
            }
            for (conn_entity, conn) in connections.iter() {
                if conn.path_entity == path_entity { commands.entity(conn_entity).despawn(); }
            }
        }
        commands.entity(path_entity).despawn();
    }
}
