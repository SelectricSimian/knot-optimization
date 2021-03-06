extern crate alga;
extern crate nalgebra;
extern crate serde_json;

extern crate knot;
extern crate rand;

use std::env::args;
use std::f64::consts::PI;
use std::f64::INFINITY;
use std::fs::File;
use std::process::exit;

use alga::general::SubsetOf;
use nalgebra::{UnitQuaternion, Vector3};

use knot::optimize_tools::{Chain, Leg, PhantomJoint, RepulsionChain};
use knot::defaults::continuous_optimization::{
    COST_PARAMS, CURVE_9_40_CHAIN_SIZE, MAX_REPULSION_STRENGTH, RATE, REPULSION,
    REPULSION_EXPONENT, REPULSION_STRENGTH, RETURN_TO_INITIAL, RETURN_TO_INITIAL_WEIGHT, STEPS,
};
use knot::geometries::curve_9_40;
use knot::isometry_adjust;
use knot::report::{JointsParity, KnotGeometry, Transform};
use knot::symmetry::{symmetries, symmetries_with_skip};

const TAU: f64 = 2.0 * PI;
const TWO_WEIGHT: f64 = 0.5;
const EPOCHS: i32 = 1000;

// Adjust the constant value as needed!
fn cooling_schedule(epoch: i32, cost_diff: f64) -> bool {
    if cost_diff > 0.0 {
        true
    } else {
        if rand::random::<f64>() < std::f64::consts::E.powf(cost_diff*(epoch as f64)* (50.0) / (EPOCHS as f64)) {
            true
        } else {
            false
        }
    }

}

fn optimize(chain: &mut RepulsionChain, steps: u32) -> f64 {
    let mut last_cost = INFINITY;
    for _ in 0..steps {
        last_cost = chain.optimize();

        if REPULSION {
            chain.repulse();
        }

        if RETURN_TO_INITIAL {
            chain.return_to_initial();
        }
    }
    last_cost
}

fn main() {
    let (mut curr_chain, symms, parity) = match args().nth(1) {
        Some(filename) => {
            let file = File::open(&filename).unwrap_or_else(|_| {
                eprintln!("Could not open file {}", filename);
                exit(1);
            });
            let geometry: KnotGeometry = serde_json::from_reader(file).unwrap_or_else(|_| {
                eprintln!("Could not parse input file");
                exit(1);
            });
            (
                RepulsionChain::new(
                    Chain::new(
                        geometry.joint_spec,
                        geometry.num_angles,
                        PhantomJoint {
                            symmetry: geometry.symmetries[1].to_isometry(),
                            index: 0,
                            leg: Leg::Incoming,
                        },
                        // post-phantom
                        PhantomJoint {
                            symmetry: geometry.symmetries[3].to_isometry(),
                            index: geometry.transforms.len() - 1,
                            leg: Leg::Outgoing,
                        },
                        geometry.cost_params,
                        RETURN_TO_INITIAL_WEIGHT,
                        RATE / 10.0,
                        isometry_adjust::Steps::new_uniform(0.000001),
                        geometry
                            .transforms
                            .iter()
                            .map(Transform::to_isometry)
                            .collect(),
                    ),
                    geometry
                        .symmetries
                        .iter()
                        .map(Transform::to_isometry)
                        .collect(),
                    REPULSION_EXPONENT,
                    REPULSION_STRENGTH,
                    MAX_REPULSION_STRENGTH,
                ),
                geometry.symmetries,
                geometry.parity,
            )
        }
        None => (
            RepulsionChain::new(
                curve_9_40::chain(
                    CURVE_9_40_CHAIN_SIZE,
                    0.7,
                    COST_PARAMS,
                    RETURN_TO_INITIAL_WEIGHT,
                    RATE,
                ),
                symmetries(3).map(|quat| quat.to_superset()).collect(),
                REPULSION_EXPONENT,
                REPULSION_STRENGTH,
                MAX_REPULSION_STRENGTH,
            ),
            symmetries_with_skip(3, 4)
                .map(|iso| Transform::from_isometry(iso.to_superset()))
                .collect(),
            JointsParity::Even,
        ),
    };
    let mut curr_cost = optimize(&mut curr_chain, STEPS);
    eprintln!("Original cost: {}", curr_cost);

    let mut steps: Vec<(usize, f64)> = Vec::new();

    let mut change: [i32; 4] = [0, 0, 0, 0];
    let mut best_cost = curr_cost;
    let mut best_epoch = 0;

    for epoch in 0..EPOCHS {

        let x: u8 = rand::random();
        let mut angle = if rand::random() {
            TAU / 16.0
        } else {
            -1.0 * TAU / 16.0
        };

        let mut offset_chain = curr_chain.clone();

        let rand_joint: usize;

        if rand::random::<f64>() > TWO_WEIGHT {
            rand_joint = (x as usize) % (curr_chain.joints.len() - 2);
            if rand_joint == 0 {
                angle = angle * 0.5;
            }
            offset_chain.joints[rand_joint] =
                offset_chain.joints[rand_joint]
                    * UnitQuaternion::from_axis_angle(&Vector3::y_axis(), angle);
            offset_chain.joints[rand_joint + 1] =
                offset_chain.joints[rand_joint]
                    * UnitQuaternion::from_axis_angle(&Vector3::y_axis(), -2.0 * angle);
            offset_chain.joints[rand_joint + 2] =
                offset_chain.joints[rand_joint]
                    * UnitQuaternion::from_axis_angle(&Vector3::y_axis(), angle);
        } else {
            rand_joint = x as usize % (curr_chain.joints.len() - 1);
            if rand_joint == 0 {
                angle = angle * 0.5;
            }
            offset_chain.joints[rand_joint] =
                offset_chain.joints[rand_joint]
                    * UnitQuaternion::from_axis_angle(&Vector3::y_axis(), angle);
            offset_chain.joints[rand_joint + 1] =
                offset_chain.joints[rand_joint]
                    * UnitQuaternion::from_axis_angle(&Vector3::y_axis(), -1.0 * angle);
        }

        let cost = optimize(&mut offset_chain, STEPS);
        let cost_diff = curr_cost - cost;
        if cooling_schedule(epoch, cost_diff) {
            curr_chain = offset_chain;
            curr_cost = cost;
            if best_cost > curr_cost {
                best_cost = curr_cost;
                best_epoch = epoch;
                eprintln!("New Best: {}", best_cost);
            }
            steps.push((rand_joint, (angle * 16.0 / TAU)));
            if epoch <= EPOCHS / 4 {
                change[0] += 1;
            } else {
                let q: usize = ((EPOCHS as f64)/(epoch as f64)).floor() as usize;
                change[q] = change[q] + 1;
            }
            eprintln!("Changed!");
            // eprintln!("{} {:+} : {} {:+}", rand_joint, (angle * 16.0 / TAU), cost, cost_diff);
        } else {
            eprintln!("Unchanged!");
            // eprintln!("{} {:+} : {} {:+}", rand_joint, (angle * 16.0 / TAU), cost, cost_diff);
        }
        eprintln!("Cost after epoch {}: {}", epoch, curr_cost);
    }

    eprintln!("\nFinal steps:");
    for &(i, offset) in &steps {
        eprintln!("{} {:+}", i, offset);
    }

    let transforms = curr_chain
        .joints
        .iter()
        .cloned()
        .map(|iso| Transform::from_isometry(iso))
        .collect::<Vec<_>>();

    let geometry = KnotGeometry {
        joint_spec: curr_chain.spec,
        num_angles: curr_chain.num_angles,
        cost_params: curr_chain.cost_params,
        parity: parity,
        symmetries: symms,
        transforms,
    };

    eprintln!("\nFinal geometry:");
    println!("{}", serde_json::to_string_pretty(&geometry).unwrap());

    eprintln!("\nChanges per Quarter of Total Steps");
    println!("{:?}", change);

    eprintln!("\nFinal Cost, Best Found Cost");
    eprintln!("{}, {} at epoch {}", curr_cost, best_cost, best_epoch);

}
