// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use iota_wallet::event::EventId;
use pyo3::{conversion::ToPyObject, exceptions, prelude::*, types::PyTuple};
use std::convert::{Into, TryFrom};

const EVENT_ID_LENGTH: usize = 32;

fn py_slice_to_event_id(id: &[u8]) -> PyResult<EventId> {
    let result = <&[u8; EVENT_ID_LENGTH]>::try_from(id);
    match result {
        Err(err) => Err(PyErr::new::<exceptions::PyTypeError, _>(err.to_string())),
        Ok(sized_id) => Ok(*sized_id),
    }
}

/// Listen to balance changes.
#[pyfunction]
pub fn on_balance_change(callback: PyObject) -> PyResult<EventId> {
    crate::block_on(async {
        Ok(iota_wallet::event::on_balance_change(move |event| {
            let event_string = serde_json::to_string(&event).unwrap();
            let gil = Python::acquire_gil();
            let py = gil.python();
            let args = PyTuple::new(py, &[event_string]);
            callback.call1(py, args).unwrap_or_else(|_| {
                PyErr::new::<exceptions::PyTypeError, _>(
                    "Unable to use the python callback function for on_balance_change()!",
                )
                .to_object(py)
            });
        })
        .await)
    })
}

/// Removes the balance change listener associated with the given identifier.
#[pyfunction]
pub fn remove_balance_change_listener(id: &[u8]) -> PyResult<()> {
    let event_id = py_slice_to_event_id(id)?;
    crate::block_on(async { iota_wallet::event::remove_balance_change_listener(&event_id).await });
    Ok(())
}

/// Listen to new messages.
#[pyfunction]
pub fn on_new_transaction(callback: PyObject) -> PyResult<EventId> {
    crate::block_on(async {
        Ok(iota_wallet::event::on_new_transaction(move |event| {
            let event_string = serde_json::to_string(&event).unwrap();
            let gil = Python::acquire_gil();
            let py = gil.python();
            let args = PyTuple::new(py, &[event_string]);
            callback.call1(py, args).unwrap_or_else(|_| {
                PyErr::new::<exceptions::PyTypeError, _>(
                    "Unable to use the python callback function for on_new_transaction()!",
                )
                .to_object(py)
            });
        })
        .await)
    })
}

/// Removes the new transaction listener associated with the given identifier.
#[pyfunction]
pub fn remove_new_transaction_listener(id: &[u8]) -> PyResult<()> {
    let event_id = py_slice_to_event_id(id)?;
    crate::block_on(async { iota_wallet::event::remove_new_transaction_listener(&event_id).await });
    Ok(())
}

/// Listen to transaction confirmation state change.
#[pyfunction]
pub fn on_confirmation_state_change(callback: PyObject) -> PyResult<EventId> {
    crate::block_on(async {
        Ok(iota_wallet::event::on_confirmation_state_change(move |event| {
            let event_string = serde_json::to_string(&event).unwrap();
            let gil = Python::acquire_gil();
            let py = gil.python();
            let args = PyTuple::new(py, &[event_string]);
            callback.call1(py, args).unwrap_or_else(|_| {
                PyErr::new::<exceptions::PyTypeError, _>(
                    "Unable to use the python callback function for on_confirmation_state_change()!",
                )
                .to_object(py)
            });
        })
        .await)
    })
}

/// Removes the new confirmation state change listener associated with the given identifier.
#[pyfunction]
pub fn remove_confirmation_state_change_listener(id: &[u8]) -> PyResult<()> {
    let event_id = py_slice_to_event_id(id)?;
    crate::block_on(async { iota_wallet::event::remove_confirmation_state_change_listener(&event_id).await });
    Ok(())
}

/// Listen to transaction reattachment.
#[pyfunction]
pub fn on_reattachment(callback: PyObject) -> PyResult<EventId> {
    crate::block_on(async {
        Ok(iota_wallet::event::on_reattachment(move |event| {
            let event_string = serde_json::to_string(&event).unwrap();
            let gil = Python::acquire_gil();
            let py = gil.python();
            let args = PyTuple::new(py, &[event_string]);
            callback.call1(py, args).unwrap_or_else(|_| {
                PyErr::new::<exceptions::PyTypeError, _>(
                    "Unable to use the python callback function for on_reattachment()!",
                )
                .to_object(py)
            });
        })
        .await)
    })
}

/// Removes the reattachment listener associated with the given identifier.
#[pyfunction]
pub fn remove_reattachment_listener(id: &[u8]) -> PyResult<()> {
    let event_id = py_slice_to_event_id(id)?;
    crate::block_on(async { iota_wallet::event::remove_reattachment_listener(&event_id).await });
    Ok(())
}

/// Listen to transaction broadcast.
#[pyfunction]
pub fn on_broadcast(callback: PyObject) -> PyResult<EventId> {
    crate::block_on(async {
        Ok(iota_wallet::event::on_broadcast(move |event| {
            let event_string = serde_json::to_string(&event).unwrap();
            let gil = Python::acquire_gil();
            let py = gil.python();
            let args = PyTuple::new(py, &[event_string]);
            callback.call1(py, args).unwrap_or_else(|_| {
                PyErr::new::<exceptions::PyTypeError, _>(
                    "Unable to use the python callback function for on_broadcast()!",
                )
                .to_object(py)
            });
        })
        .await)
    })
}

/// Removes the broadcast listener associated with the given identifier.
#[pyfunction]
pub fn remove_broadcast_listener(id: &[u8]) -> PyResult<()> {
    let event_id = py_slice_to_event_id(id)?;
    crate::block_on(async { iota_wallet::event::remove_broadcast_listener(&event_id).await });
    Ok(())
}

/// Listen to errors.
#[pyfunction]
pub fn on_error(callback: PyObject) -> PyResult<EventId> {
    Ok(iota_wallet::event::on_error(move |event| {
        let event_string = serde_json::to_string(&event).unwrap();
        let gil = Python::acquire_gil();
        let py = gil.python();
        let args = PyTuple::new(py, &[event_string]);
        callback.call1(py, args).unwrap_or_else(|_| {
            PyErr::new::<exceptions::PyTypeError, _>("Unable to use the python callback function for on_error()!")
                .to_object(py)
        });
    }))
}

/// Removes the error listener associated with the given identifier.
#[pyfunction]
pub fn remove_error_listener(id: &[u8]) -> PyResult<()> {
    let event_id = py_slice_to_event_id(id)?;
    iota_wallet::event::remove_error_listener(&event_id);
    Ok(())
}

/// Listen to stronghold status change events.
#[pyfunction]
pub fn on_stronghold_status_change(callback: PyObject) -> PyResult<EventId> {
    crate::block_on(async {
        Ok(iota_wallet::event::on_stronghold_status_change(move |event| {
            let event_string = serde_json::to_string(&event).unwrap();
            let gil = Python::acquire_gil();
            let py = gil.python();
            let args = PyTuple::new(py, &[event_string]);
            callback.call1(py, args).unwrap_or_else(|_| {
                PyErr::new::<exceptions::PyTypeError, _>(
                    "Unable to use the python callback function for on_stronghold_status_change()!",
                )
                .to_object(py)
            });
        })
        .await)
    })
}

/// Removes the stronghold status change listener associated with the given identifier.
#[pyfunction]
pub fn remove_stronghold_status_change_listener(id: &[u8]) -> PyResult<()> {
    let event_id = py_slice_to_event_id(id)?;
    crate::block_on(async { iota_wallet::event::remove_stronghold_status_change_listener(&event_id).await });
    Ok(())
}
