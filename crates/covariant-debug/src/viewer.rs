//! 3D viewer for step-by-step debug visualization using kiss3d.

use std::cell::RefCell;
use std::rc::Rc;

use kiss3d::camera::ArcBall;
use kiss3d::event::{Action, Key, WindowEvent};
use kiss3d::light::Light;
use kiss3d::resource::Mesh as KissMesh;
use kiss3d::scene::SceneNode;
use kiss3d::window::Window;
use nalgebra::{Point3, Vector3};

use covariant_geom::GeomKernel;

use crate::trace::DebugSession;

/// Default tessellation tolerance for viewer meshes.
const VIEW_TOLERANCE: f64 = 0.05;

/// Color for the currently active step (blue).
const ACTIVE_COLOR: (f32, f32, f32) = (0.3, 0.5, 1.0);
/// Color for previously completed steps (gray).
const PASSIVE_COLOR: (f32, f32, f32) = (0.6, 0.6, 0.6);

/// Convert a covariant-geom `Solid` to a kiss3d-compatible `Mesh`.
///
/// Returns `None` if the mesh is empty or has too many vertices for u16 indices.
fn solid_to_kiss3d_mesh(
    solid: &covariant_geom::Solid,
    kernel: &dyn GeomKernel,
) -> Option<Rc<RefCell<KissMesh>>> {
    let mesh = kernel.tessellate(solid, VIEW_TOLERANCE);
    if mesh.is_empty() {
        return None;
    }

    let positions = mesh.positions();
    let faces = mesh.tri_faces();

    // kiss3d uses u16 for face indices.
    if positions.len() > u16::MAX as usize {
        eprintln!(
            "warning: mesh has {} vertices, exceeding u16 limit; skipping",
            positions.len()
        );
        return None;
    }

    let coords: Vec<Point3<f32>> = positions
        .iter()
        .map(|p| Point3::new(p[0] as f32, p[1] as f32, p[2] as f32))
        .collect();

    let indices: Vec<Point3<u16>> = faces
        .iter()
        .map(|f| Point3::new(f[0] as u16, f[1] as u16, f[2] as u16))
        .collect();

    let kiss_mesh = KissMesh::new(coords, indices, None, None, false);
    Some(Rc::new(RefCell::new(kiss_mesh)))
}

/// Convert a byte offset to (line, col) for display.
fn offset_to_line_col(source: &str, offset: u32) -> (usize, usize) {
    let offset = offset as usize;
    let mut line = 1;
    let mut col = 1;
    for (i, ch) in source.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

/// Format step info for display.
fn format_step_info(session: &DebugSession, step_idx: usize) -> String {
    let step = &session.steps[step_idx];
    let (line, col) = offset_to_line_col(&session.source, step.span.start);
    let label = step
        .label
        .as_deref()
        .unwrap_or("(unlabeled)");
    format!(
        "Step {}/{}: {} ({}:{}:{})",
        step_idx + 1,
        session.step_count(),
        label,
        session.file_path,
        line,
        col,
    )
}

/// Launch the interactive 3D viewer for a debug session.
///
/// Opens a kiss3d window showing step-by-step geometry.
/// - **Left/Right arrows**: previous/next step
/// - **Home/End**: first/last step
/// - **Q/Escape**: quit
pub fn launch_viewer(session: &DebugSession, kernel: &dyn GeomKernel) {
    if session.steps.is_empty() {
        eprintln!("No geometry steps to display.");
        return;
    }

    // Pre-tessellate all step solids into kiss3d meshes.
    let meshes: Vec<Option<Rc<RefCell<KissMesh>>>> = session
        .steps
        .iter()
        .map(|step| solid_to_kiss3d_mesh(&step.solid, kernel))
        .collect();

    let mut window = Window::new_with_size("COVARIANT Debug Viewer", 1024, 768);
    window.set_light(Light::StickToCamera);
    window.set_background_color(0.15, 0.15, 0.18);

    // Set up camera looking at the origin from a reasonable distance.
    let eye = Point3::new(50.0, 50.0, 50.0);
    let at = Point3::new(0.0, 0.0, 0.0);
    let mut camera = ArcBall::new(eye, at);

    let mut current_step: usize = 0;
    let total = session.step_count();

    // Add all mesh scene nodes (initially hidden except step 0).
    let mut scene_nodes: Vec<Option<SceneNode>> = Vec::with_capacity(total);
    for (i, mesh_opt) in meshes.iter().enumerate() {
        match mesh_opt {
            Some(mesh) => {
                let mut node = window.add_mesh(
                    Rc::clone(mesh),
                    Vector3::new(1.0, 1.0, 1.0),
                );
                if i == 0 {
                    node.set_color(ACTIVE_COLOR.0, ACTIVE_COLOR.1, ACTIVE_COLOR.2);
                    node.set_visible(true);
                } else {
                    node.set_color(PASSIVE_COLOR.0, PASSIVE_COLOR.1, PASSIVE_COLOR.2);
                    node.set_visible(false);
                }
                node.enable_backface_culling(false);
                scene_nodes.push(Some(node));
            }
            None => {
                scene_nodes.push(None);
            }
        }
    }

    // Print initial step info.
    let info = format_step_info(session, current_step);
    eprintln!("{info}");
    window.set_title(&info);

    while window.render_with_camera(&mut camera) {
        let mut step_changed = false;

        for mut event in window.events().iter() {
            if let WindowEvent::Key(key, Action::Press, _) = event.value {
                match key {
                    Key::Q | Key::Escape => return,
                    Key::Right => {
                        if current_step + 1 < total {
                            current_step += 1;
                            step_changed = true;
                        }
                        event.inhibited = true;
                    }
                    Key::Left => {
                        if current_step > 0 {
                            current_step -= 1;
                            step_changed = true;
                        }
                        event.inhibited = true;
                    }
                    Key::Home => {
                        if current_step != 0 {
                            current_step = 0;
                            step_changed = true;
                        }
                        event.inhibited = true;
                    }
                    Key::End => {
                        if current_step != total - 1 {
                            current_step = total - 1;
                            step_changed = true;
                        }
                        event.inhibited = true;
                    }
                    _ => {}
                }
            }
        }

        if step_changed {
            // Update visibility: show steps 0..=current_step.
            // Current step in active color, others in passive.
            for (i, node_opt) in scene_nodes.iter_mut().enumerate() {
                if let Some(node) = node_opt {
                    if i <= current_step {
                        node.set_visible(true);
                        if i == current_step {
                            node.set_color(ACTIVE_COLOR.0, ACTIVE_COLOR.1, ACTIVE_COLOR.2);
                        } else {
                            node.set_color(PASSIVE_COLOR.0, PASSIVE_COLOR.1, PASSIVE_COLOR.2);
                        }
                    } else {
                        node.set_visible(false);
                    }
                }
            }

            let info = format_step_info(session, current_step);
            eprintln!("{info}");
            window.set_title(&info);
        }
    }
}

#[cfg(test)]
mod tests {
    // Viewer tests require a display server and cannot run in CI.
    // The module compiles successfully as a smoke test.

    #[test]
    fn viewer_module_compiles() {
        // Smoke test: this function existing means the module compiles.
        assert!(true);
    }
}
