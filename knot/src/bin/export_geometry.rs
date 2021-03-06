extern crate alga;
extern crate nalgebra;
extern crate serde;
extern crate serde_json;

extern crate knot;

use std::env::args;
use std::fs::File;
use std::process::exit;

use alga::general::SubsetOf;
use nalgebra::Isometry3;

use knot::joint::{at_angles, discrete_symmetric_angles};
use knot::report::{
    complete_report, complete_reports, CompleteKnotReports, JointsParity, KnotGeometry, Transform,
};
use knot::symmetry::symmetries_with_skip;

fn main() {
    let filename = args().nth(1).unwrap_or_else(|| {
        eprintln!("Expected a single input file");
        exit(1);
    });
    let index = args()
        .nth(2)
        .unwrap_or_else(|| {
            eprintln!("Expected a result index");
            exit(1);
        }).parse::<usize>()
        .unwrap_or_else(|_| {
            eprintln!("Index must be an integer");
            exit(1)
        });
    let file = File::open(&filename).unwrap_or_else(|_| {
        eprintln!("Could not open file {}", filename);
        exit(1);
    });
    let reports: CompleteKnotReports =
        complete_reports(serde_json::from_reader(file).unwrap_or_else(|_| {
            eprintln!("Could not parse input file");
            exit(1);
        }));

    if !(index < reports.knots.len()) {
        eprintln!(
            "Index out of bounds -- only {} reports",
            reports.knots.len()
        );
        exit(1);
    }

    let knot = complete_report(&reports, index);

    let mut isometries = Vec::new();

    match reports.parity {
        JointsParity::Even => {}
        JointsParity::Odd => isometries.push(reports.joint_spec.origin_to_symmetric()),
    }

    isometries.extend(at_angles(
        discrete_symmetric_angles(
            reports.joint_spec,
            reports.num_angles,
            reports.parity,
            knot.angles.iter().cloned().map(|angle| angle as i32),
        ),
        match reports.parity {
            JointsParity::Even => Isometry3::identity(),
            JointsParity::Odd => {
                reports.joint_spec.origin_to_symmetric() * reports.joint_spec.origin_to_out()
            }
        },
    ));

    let adjust_trans = knot.symmetry_adjust.transform();

    let transforms = isometries
        .iter()
        .cloned()
        .map(|iso| Transform::from_isometry(adjust_trans * iso))
        .collect::<Vec<_>>();

    let symms = symmetries_with_skip(reports.symmetry_count, reports.symmetry_skip)
        .map(|quat| Transform::from_isometry(quat.to_superset()))
        .collect::<Vec<_>>();

    let geometry = KnotGeometry {
        joint_spec: reports.joint_spec,
        num_angles: reports.num_angles,
        cost_params: reports.cost_params,
        parity: reports.parity,
        symmetries: symms,
        transforms,
    };

    println!("{}", serde_json::to_string_pretty(&geometry).unwrap());
}
