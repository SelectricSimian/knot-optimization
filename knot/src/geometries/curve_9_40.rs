use std::f64::consts::PI;

use nalgebra::{UnitQuaternion, Vector3};
use alga::general::SubsetOf;

use defaults;
use isometry_adjust as iso_adj;
use continuous_optimize::{Leg, PhantomJoint, Chain};
use cost::CostParams;
use symmetry::adjacent_symmetry;

use geometries::spherical::spherical;
use geometries::from_curve::from_curve;

const TAU: f64 = 2.0 * PI;

pub fn chain(chain_size: usize, scale: f64, cost_params: CostParams, descent_rate: f64) -> Chain {
    Chain {
        spec: defaults::joint_spec(),
        num_angles: defaults::NUM_ANGLES,
        pre_phantom: PhantomJoint {
            symmetry: UnitQuaternion::from_axis_angle(&Vector3::x_axis(), PI).to_superset(),
            index: 0,
            leg: Leg::Incoming,
        },
        post_phantom: PhantomJoint {
            symmetry: adjacent_symmetry(3, 1).to_superset(),
            index: chain_size - 1,
            leg: Leg::Outgoing,
        },
        cost_params: cost_params,
        descent_rate,
        steps: iso_adj::Steps::new_uniform(0.000001),
        joints: from_curve(chain_size, 0.0, 1.0, |t| {
            let theta = 2.0 * TAU / 3.0 * t;
            let phi = 1.0 * (TAU / 2.0 * t).sin();
            let rho = 7.0 + 2.5 * (TAU / 2.0 * t).cos();
            spherical(theta, phi, scale * rho)
        }).collect(),
    }
}