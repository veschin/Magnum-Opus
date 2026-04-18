use magnum_opus::core::*;

#[test]
fn empty_core_ticks() {
    let mut app = Harness::new().build();
    assert_eq!(app.world().resource::<Tick>().0, 0);
    app.update();
    assert_eq!(app.world().resource::<Tick>().0, 1);
    app.update();
    app.update();
    assert_eq!(app.world().resource::<Tick>().0, 3);
}

#[test]
fn core_registers_shared_resources() {
    let app = Harness::new().build();
    assert!(app.world().contains_resource::<Tick>());
    assert!(app.world().contains_resource::<ModuleRegistry>());
    assert!(app.world().contains_resource::<MetricsRegistry>());
    assert_eq!(app.world().resource::<ModuleRegistry>().len(), 1);
    assert_eq!(app.world().resource::<MetricsRegistry>().len(), 0);
}
