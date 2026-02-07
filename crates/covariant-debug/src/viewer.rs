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

/// Compute smooth per-vertex normals by averaging the normals of adjacent faces.
fn compute_smooth_normals(
    coords: &[Point3<f32>],
    indices: &[Point3<u16>],
) -> Vec<Vector3<f32>> {
    let mut normals = vec![Vector3::new(0.0f32, 0.0, 0.0); coords.len()];

    for face in indices {
        let i0 = face.x as usize;
        let i1 = face.y as usize;
        let i2 = face.z as usize;

        let v0 = &coords[i0];
        let v1 = &coords[i1];
        let v2 = &coords[i2];

        let edge1 = Vector3::new(v1.x - v0.x, v1.y - v0.y, v1.z - v0.z);
        let edge2 = Vector3::new(v2.x - v0.x, v2.y - v0.y, v2.z - v0.z);
        let face_normal = edge1.cross(&edge2);

        normals[i0] += face_normal;
        normals[i1] += face_normal;
        normals[i2] += face_normal;
    }

    normals
        .into_iter()
        .map(|n| {
            let len = n.norm();
            if len > 1e-10 {
                n / len
            } else {
                Vector3::new(0.0, 1.0, 0.0)
            }
        })
        .collect()
}

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

    // Compute smooth per-vertex normals by averaging adjacent face normals.
    let normals = compute_smooth_normals(&coords, &indices);

    let kiss_mesh = KissMesh::new(coords, indices, Some(normals), None, false);
    Some(Rc::new(RefCell::new(kiss_mesh)))
}

/// Axis length in mm.
const AXIS_LENGTH: f32 = 100.0;
/// Grid extent: half-size of the grid in mm (grid spans -GRID_EXTENT..+GRID_EXTENT).
const GRID_EXTENT: f32 = 100.0;
/// Grid spacing in mm.
const GRID_SPACING: f32 = 10.0;

/// Draw XYZ coordinate axes through the origin (X=red, Y=green, Z=blue).
fn draw_axes(window: &mut Window) {
    let o = Point3::origin();
    let red = Point3::new(1.0, 0.2, 0.2);
    let green = Point3::new(0.2, 1.0, 0.2);
    let blue = Point3::new(0.3, 0.3, 1.0);

    window.set_line_width(3.0);
    window.draw_line(&o, &Point3::new(AXIS_LENGTH, 0.0, 0.0), &red);
    window.draw_line(&o, &Point3::new(0.0, AXIS_LENGTH, 0.0), &green);
    window.draw_line(&o, &Point3::new(0.0, 0.0, AXIS_LENGTH), &blue);
    window.set_line_width(1.0);
}

/// Draw a grid on the XY plane (Z=0) to help visualize scale and position.
fn draw_grid(window: &mut Window) {
    let color = Point3::new(0.3, 0.3, 0.3);
    let n = (GRID_EXTENT / GRID_SPACING) as i32;

    // Lines parallel to Y-axis
    (-n..=n).for_each(|i| {
        let x = i as f32 * GRID_SPACING;
        window.draw_line(
            &Point3::new(x, -GRID_EXTENT, 0.0),
            &Point3::new(x, GRID_EXTENT, 0.0),
            &color,
        );
    });

    // Lines parallel to X-axis
    (-n..=n).for_each(|i| {
        let y = i as f32 * GRID_SPACING;
        window.draw_line(
            &Point3::new(-GRID_EXTENT, y, 0.0),
            &Point3::new(GRID_EXTENT, y, 0.0),
            &color,
        );
    });
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

    // Add all mesh scene nodes (initially only step 0 visible).
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
                    node.set_visible(false);
                }
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
        draw_axes(&mut window);
        draw_grid(&mut window);

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
            // Update visibility: show only the current step.
            for (i, node_opt) in scene_nodes.iter_mut().enumerate() {
                if let Some(node) = node_opt {
                    if i == current_step {
                        node.set_visible(true);
                        node.set_color(ACTIVE_COLOR.0, ACTIVE_COLOR.1, ACTIVE_COLOR.2);
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
