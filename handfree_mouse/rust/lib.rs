// HandFree Mouse - Rust FFI Bindings
// Python-Rust bridge for high-performance mouse control

use pyo3::prelude::*;
use pyo3::types::PyDict;
use enigo::{Enigo, Mouse, Button, Coordinate, Direction};
use std::sync::{Arc, Mutex};

/// Global mouse controller instance
static MOUSE_CONTROLLER: once_cell::sync::Lazy<Arc<Mutex<Enigo>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(Enigo::new(&enigo::Settings::default()).unwrap())));

/// Move mouse cursor to absolute position
#[pyfunction]
fn move_cursor(x: i32, y: i32) -> PyResult<()> {
    let mut enigo = MOUSE_CONTROLLER.lock().unwrap();
    enigo.move_mouse(x, y, Coordinate::Abs).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to move mouse: {}", e))
    })?;
    Ok(())
}

/// Move mouse cursor by relative amount
#[pyfunction]
fn move_cursor_relative(dx: i32, dy: i32) -> PyResult<()> {
    let mut enigo = MOUSE_CONTROLLER.lock().unwrap();
    enigo.move_mouse(dx, dy, Coordinate::Rel).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to move mouse: {}", e))
    })?;
    Ok(())
}

/// Perform left mouse click
#[pyfunction]
fn left_click() -> PyResult<()> {
    let mut enigo = MOUSE_CONTROLLER.lock().unwrap();
    enigo.button(Button::Left, enigo::Direction::Click).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to click: {}", e))
    })?;
    Ok(())
}

/// Perform right mouse click
#[pyfunction]
fn right_click() -> PyResult<()> {
    let mut enigo = MOUSE_CONTROLLER.lock().unwrap();
    enigo.button(Button::Right, enigo::Direction::Click).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to click: {}", e))
    })?;
    Ok(())
}

/// Press mouse button down
#[pyfunction]
fn mouse_down(button: &str) -> PyResult<()> {
    let mut enigo = MOUSE_CONTROLLER.lock().unwrap();
    let btn = match button {
        "left" => Button::Left,
        "right" => Button::Right,
        "middle" => Button::Middle,
        _ => return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>("Invalid button")),
    };
    enigo.button(btn, Direction::Press).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to press button: {}", e))
    })?;
    Ok(())
}

/// Release mouse button
#[pyfunction]
fn mouse_up(button: &str) -> PyResult<()> {
    let mut enigo = MOUSE_CONTROLLER.lock().unwrap();
    let btn = match button {
        "left" => Button::Left,
        "right" => Button::Right,
        "middle" => Button::Middle,
        _ => return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>("Invalid button")),
    };
    enigo.button(btn, Direction::Release).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to release button: {}", e))
    })?;
    Ok(())
}

/// Scroll mouse wheel
#[pyfunction]
fn scroll(amount: i32, axis: &str) -> PyResult<()> {
    let mut enigo = MOUSE_CONTROLLER.lock().unwrap();
    match axis {
        "vertical" => {
            enigo.scroll(amount, enigo::Axis::Vertical).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to scroll: {}", e))
            })?;
        }
        "horizontal" => {
            enigo.scroll(amount, enigo::Axis::Horizontal).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to scroll: {}", e))
            })?;
        }
        _ => return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>("Invalid axis")),
    }
    Ok(())
}

/// Get current mouse position
#[pyfunction]
fn get_mouse_position() -> PyResult<(i32, i32)> {
    let enigo = MOUSE_CONTROLLER.lock().unwrap();
    let location = enigo.location().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to get position: {}", e))
    })?;
    Ok((location.0, location.1))
}

/// Python module definition
#[pymodule]
fn handfree_mouse_rust(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(move_cursor, m)?)?;
    m.add_function(wrap_pyfunction!(move_cursor_relative, m)?)?;
    m.add_function(wrap_pyfunction!(left_click, m)?)?;
    m.add_function(wrap_pyfunction!(right_click, m)?)?;
    m.add_function(wrap_pyfunction!(mouse_down, m)?)?;
    m.add_function(wrap_pyfunction!(mouse_up, m)?)?;
    m.add_function(wrap_pyfunction!(scroll, m)?)?;
    m.add_function(wrap_pyfunction!(get_mouse_position, m)?)?;
    Ok(())
}
