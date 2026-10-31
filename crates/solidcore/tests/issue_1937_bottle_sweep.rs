//! Characterization of the unresolved bottle sweep defect (#1937).
//! These tests pin the invalid default output; they do not claim a repair.
use solidcore::geom::attribute::ObjectKey;
use solidcore::sweep::topology_sweep::store::SweepIdentities;
use solidcore::topology::{BrepId, EdgeId, RegionId};
use solidcore::{Brep, Result};

use solidcore::geom::attribute::AttributeStore;
use solidcore::sweep::topology_sweep::{
    access::SweepAccess, engine::SweepTargets, geometry::SweepGeometry, store::SweepStore,
    TopologySweep,
};
use solidcore::sweep::trans_sweep_geometry::TranslationalSweepGeometry;
use solidcore::topology::api_files::read_brep_from_file;
use solidcore::topology::brep::p10::TopologyEnv;
use solidcore::topology::brep::validation::{check_brep, CheckLevel};
use solidcore::topology::topology::NotifyRecords;
use solidcore::{Kernel, Vec3};
#[test]
#[ignore = "requires the pinned upstream bottle.smb fixture"]
fn bottle_sweep_reproduces_native_failures() {
    let mut k = Kernel::default();
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../upstream/TestFiles/pt_TestFiles/Fillet/bottle.smb");
    let bytes = std::fs::read(path).unwrap();
    let mut attributes = AttributeStore::new();
    let mut records = NotifyRecords::new();
    let key_of = |e| WholeIdentities.topology(BrepId::from_index(0), e);
    let mut env = TopologyEnv {
        all_attributes: &[],
        attributes: &mut attributes,
        key_of: &key_of,
        records: &mut records,
        next_key: 1 << 50,
    };
    let mut b = read_brep_from_file(&mut k, Some(bytes), true, false, &mut env).unwrap();
    b.refresh_cached_topology();
    let before = check_brep(&mut k, &mut b, CheckLevel::Level0).unwrap();
    eprintln!(
        "BEFORE valid={} lines={:?}",
        before.is_valid(),
        before.lines()
    );
    b.set_key(key_of(solidcore::error::EntityRef::Brep(BrepId::from_index(0))).key);
    let identities = WholeIdentities;
    let mut next = 0;
    let mut store = SweepStore::new(&identities, &mut records, &mut attributes, &mut next);
    let context = store.register_kernel(&mut k).unwrap();
    let id = store.register_brep(context, &mut b).unwrap();
    let region = store.infinite_region(id).unwrap();
    let g = SweepGeometry::Translation(TranslationalSweepGeometry::new(Vec3::new(0., 0., 4.)));
    let mut sweep = TopologySweep {
        do_stitching: false,
        do_merge: false,
        repetitions: 1,
        ..TopologySweep::default()
    };
    let result = sweep.do_sweep(
        &mut store,
        &g,
        Some(id),
        Some((id, region)),
        &mut SweepTargets::default(),
    );
    eprintln!("SWEEP {result:?}");
    result.unwrap();
    drop(store);
    let after = check_brep(&mut k, &mut b, CheckLevel::Level0).unwrap();
    eprintln!("AFTER valid={} lines={:?}", after.is_valid(), after.lines());
    assert_eq!(before.pointers_status, 1000);
    assert_eq!(before.assert_valid, Some(true));
    assert_eq!(before.reports.reports().len(), 8);
    assert_eq!(after.pointers_status, 1000);
    assert_eq!(after.assert_valid, Some(false));
    assert_eq!(after.reports.reports().len(), 6);
    let introduced: Vec<_> = after
        .reports
        .reports()
        .iter()
        .filter(|r| r.reporting_type != 16011)
        .map(|r| (r.reporting_type, r.test_index))
        .collect();
    assert_eq!(introduced, [(15001, 1), (15001, 6), (12225, 2)]);
    // Validation populates derived UV curves. A second check of the unchanged body
    // exposes an additional loop-orientation failure in pointer validation.
    let repeated = check_brep(&mut k, &mut b, CheckLevel::Level0).unwrap();
    eprintln!("REPEATED {:?}", repeated.lines());
    assert_eq!(repeated.pointers_status, 1001);
    assert_eq!(repeated.assert_valid, None);
    assert_eq!(
        repeated
            .pointer_reports
            .reports()
            .iter()
            .map(|r| (r.reporting_type, r.test_index))
            .collect::<Vec<_>>(),
        [(12225, 2), (12225, 3), (12225, 4)]
    );
}

#[test]
fn near_parallel_generator_returns_invalid_surface() {
    use solidcore::curve::{curve::Curve, line::Line};
    use solidcore::geom::{assert_array::AssertArray, extent1d::Extent1d};
    let mut k = Kernel::default();
    let mut b = Brep::new_with_infinite_region().unwrap();
    let c = Curve::from(
        Line::create_line_segment(
            3,
            Vec3::new(-0.30118110236220502, 0., 4.3331526241915306),
            Vec3::new(-0.30118297798525001, 0., 4.2633059315015203),
            None,
        )
        .unwrap(),
    );
    let edge = solidcore::topology::brep::p5::create_wire_edge_from_curve(
        &mut k,
        &mut b,
        c,
        Extent1d::from_min_max(0., 1.),
    )
    .unwrap();
    let mut g = TranslationalSweepGeometry::new(Vec3::new(0., 0., 4.));
    g.base.make_analytics = false;
    let result = g.edge_sweep_higher(&k, &b, edge);
    eprintln!("REDUCED sweep={:?}", result.as_ref().map(|_| ()));
    let swept = result.unwrap();
    {
        let mut reports = AssertArray::new();
        let valid = swept
            .surface
            .assert_valid(
                &k,
                Some(&mut reports),
                0,
                solidcore::geom::assert_array::ObjectKey(1),
            )
            .unwrap();
        eprintln!("REDUCED valid={valid} reports={reports:?}");
        assert!(!valid);
        assert_eq!(
            reports
                .reports()
                .iter()
                .map(|r| (r.reporting_type, r.test_index))
                .collect::<Vec<_>>(),
            [(15001, 1), (15001, 6)]
        );
    }
}

struct WholeIdentities;
impl SweepIdentities for WholeIdentities {
    fn topology(
        &self,
        b: BrepId,
        e: solidcore::error::EntityRef,
    ) -> solidcore::geom::object::ObjectRef {
        use solidcore::error::EntityRef as R;
        // (kind, index, native type code)
        let (kind, index, type_code) = match e {
            R::Brep(i) => (0_u64, i.index(), 16011),
            R::Shell(i) => (1, i.index(), 16013),
            R::Face(i) => (2, i.index(), 16002),
            R::Loop(i) => (3, i.index(), 16003),
            R::Edge(i) => (4, i.index(), 16016),
            R::Vertex(i) => (5, i.index(), 16021),
            R::Region(i) => (6, i.index(), 16012),
            R::Faceuse(i) => (7, i.index(), 16022),
            R::Loopuse(i) => (8, i.index(), 16004),
            R::Edgeuse(i) => (9, i.index(), 16005),
            R::Vertexuse(i) => (10, i.index(), 16006),
        };
        solidcore::geom::object::ObjectRef {
            type_code,
            key: ObjectKey((u64::from(b.index()) << 40) | (kind << 32) | u64::from(index)),
        }
    }
    fn curve(
        &self,
        b: BrepId,
        e: EdgeId,
        _: &solidcore::curve::curve::Curve,
    ) -> solidcore::geom::object::ObjectRef {
        solidcore::geom::object::ObjectRef {
            type_code: 11001,
            key: ObjectKey((u64::from(b.index()) << 40) | (20 << 32) | u64::from(e.index())),
        }
    }
    /// A curve the stitch allocates when it splits an edge (`MakeVertexSplitEdge`'s
    /// curve copy) is named, as every edge curve here, by its edge: the native copy
    /// is a new object of its own, distinct from every other curve.
    fn new_curve(
        &self,
        b: BrepId,
        e: EdgeId,
        type_code: i64,
    ) -> Result<solidcore::geom::object::ObjectRef> {
        Ok(solidcore::geom::object::ObjectRef {
            type_code,
            key: ObjectKey((u64::from(b.index()) << 40) | (20 << 32) | u64::from(e.index())),
        })
    }
    fn owned_auxiliary_objects(&self, _: BrepId, _: &Brep) -> Vec<ObjectKey> {
        Vec::new()
    }
    /// The merge's two source lists of a primary region: handles outside every
    /// topology and curve kind above.
    fn source_lists(
        &self,
        b: BrepId,
        r: RegionId,
    ) -> Result<[solidcore::geom::object::ObjectRef; 2]> {
        Ok([21_u64, 22].map(|kind| solidcore::geom::object::ObjectRef {
            type_code: 0,
            key: ObjectKey((u64::from(b.index()) << 40) | (kind << 32) | u64::from(r.index())),
        }))
    }
}
