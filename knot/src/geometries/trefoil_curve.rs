extern crate bspline;
use std::f64::consts::PI;

use alga::general::SubsetOf;
use nalgebra::{UnitQuaternion, Vector3};

use optimize_tools::{Chain, Leg, PhantomJoint};
use cost::CostParams;
use defaults;
use isometry_adjust as iso_adj;
use symmetry::adjacent_symmetry;

use geometries::from_spline::from_spline;
use geometries::trefoil_spline::generate_trefoil;

pub fn chain(
    scale: f32,
    cost_params: CostParams,
    return_to_initial_weight: f64,
    descent_rate: f64,
) -> Chain {
    let spline_iter = from_spline(
        2.2, // arc length step
        generate_trefoil, // bspline generator
        3, // symmetry
        scale,  // scale
    );
    let chain_size = spline_iter.0;

    Chain::new(
        // spec
        defaults::joint_spec(),
        // num angles
        defaults::NUM_ANGLES,
        // pre-phantom
        PhantomJoint {
            symmetry: UnitQuaternion::from_axis_angle(&Vector3::x_axis(), PI).to_superset(),
            index: 0,
            leg: Leg::Incoming,
        },
        // post-phantom
        PhantomJoint {
            symmetry: adjacent_symmetry(3, 4).to_superset(),
            index: chain_size - 1,
            leg: Leg::Outgoing,
        },
        // cost params
        cost_params,
        // 'return to initial' weight
        return_to_initial_weight,
        // descent rate
        descent_rate,
        // steps
        iso_adj::Steps::new_uniform(0.000001),
        // joints
        spline_iter.1.collect(),
    )
}
