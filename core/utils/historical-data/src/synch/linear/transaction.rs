// Copyright 2019 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! Linear arrangement of historical data with transactional
//! support.
//!
//!
//! # Global state
//!
//! The only global state is a counter of overlayed transaction layer.
//! Committing or discarding a layer must use this counter.
//! 
//! # Local state
//!
//! Local state is either a committed state (this is a single first independant level
//! of transaction) or a reference to the transaction counter in use in time of creation.

use rstd::vec::Vec;
use crate::PruneResult;

/// Global state is a simple counter to the current overlay layer index.
#[derive(Debug, Clone)]
#[cfg_attr(any(test, feature = "test-helpers"), derive(PartialEq))]
pub struct States(usize);
	
impl Default for States {
	fn default() -> Self {
		States(0)
	}
}

impl States {

	/// Build any state for testing only.
	#[cfg(any(test, feature = "test-helpers"))]
	pub fn test_state(
		current_layer_number: usize,
	) -> Self {
		States(current_layer_number)
	}

	/// Discard prospective changes to state.
	/// It does not reverts actual values. 
	/// A subsequent synchronisation of stored values is needed.
	pub fn discard_prospective(&mut self) {
		if self.0 > 0 {
			self.0 -= 1;
		}
	}

	/// Update a value to a new prospective.
	pub fn apply_discard_prospective(&self) {
		unimplemented!("TODO History as mut param");
	}

	/// Commit prospective changes to state.
	/// A subsequent synchronisation of stored values is needed.
	pub fn commit_prospective(&mut self) {
		if self.0 > 0 {
			self.0 -= 1;
		}
	}

	/// Update a value to a new prospective.
	/// Multiple commit can be applied at the same time.
	pub fn apply_commit_prospective(&self) {
		unimplemented!("TODO History as mut param");
	}


	/// Create a new transactional layer.
	pub fn start_transaction(&mut self) {
		self.0 += 1;
	}

	/// Discard a transactional layer.
	/// It does not reverts actual values.
	/// A subsequent synchronisation of stored values is needed.
	pub fn discard_transaction(&mut self) {
		if self.0 > 0 {
			self.0 -= 1;
		}
	}

	/// Update a value to previous transaction.
	/// Multiple discard can be applied at the same time.
	/// Returns true if value is still needed.
	pub fn apply_discard_transaction(&self) -> PruneResult {
		unimplemented!("TODO History as mut param");
	}

	/// Discard a transactional layer.
	/// It does not reverts actual values.
	/// A subsequent synchronisation of stored values is needed.
	pub fn commit_transaction(&mut self) {
		if self.0 > 0 {
			self.0 -= 1;
		}
	}

	/// Update a value to be the best historical value
	/// after one or more `commit_transaction` calls.
	/// Multiple discard can be applied at the same time.
	/// Returns true if value is still needed.
	pub fn apply_commit_transaction(&self) -> PruneResult {
		unimplemented!("TODO History as mut param");
	}

}

/// Possible state for a historical value, committed
/// is not touched by transactional action, transaction
/// stored the transaction index of insertion.
#[derive(Debug, Clone)]
#[cfg_attr(any(test, feature = "test-helpers"), derive(PartialEq))]
pub enum State {
	Committed,
	Transaction(usize),
}
impl State {
	fn transaction_index(&self) -> Option<usize> {
		if let &State::Transaction(ix) = self {
			Some(ix)
		} else {
			None
		}
	}
}
/// An entry at a given history height.
pub type HistoricalValue<V> = crate::HistoricalValue<V, State>;

/// History of value and their state.
#[derive(Debug, Clone)]
#[cfg_attr(any(test, feature = "test-helpers"), derive(PartialEq))]
pub struct History<V>(pub(crate) Vec<HistoricalValue<V>>);

impl<V> Default for History<V> {
	fn default() -> Self {
		History(Default::default())
	}
}

impl<V> History<V> {
	/// Set a value, it uses a global state as parameter.
	pub fn set(&mut self, states: &States, value: V) {
		if let Some(v) = self.0.last_mut() {
			debug_assert!(v.index.transaction_index().unwrap_or(0) <= states.0,
				"History expects \
				only new values at the latest state, some state has not \
				synchronized properly");
			if v.index.transaction_index() == Some(states.0) {
				v.value = value;
				return;
			}
		}
		self.0.push(HistoricalValue {
			value,
			index: State::Transaction(states.0),
		});
	}

	/// Access to the latest pending value.
	pub fn get(&self) -> Option<&V> {
		self.0.last().map(|h| &h.value)
	}

	/// Get latest value, consuming the historical data.
	pub fn into_pending(mut self) -> Option<V> {
		if let Some(v) = self.0.pop() {
			Some(v.value)
		} else {
			None
		}
	}

	#[cfg(any(test, feature = "test-helpers"))]
	pub fn get_prospective(&self) -> Option<&V> {
		match self.0.get(0) {
			Some(HistoricalValue {
				value: _,
				index: State::Committed,
			}) => {
				if let Some(HistoricalValue {
					value,
					index: State::Transaction(_),
				}) = self.0.get(1) {
					Some(&value)
				} else {
					None
				}
			},
			Some(HistoricalValue {
				value,
				index: State::Transaction(_),
			}) => Some(&value),
			None => None,
		}
	}

	#[cfg(any(test, feature = "test-helpers"))]
	pub fn get_committed(&self) -> Option<&V> {
		if let Some(HistoricalValue {
					value,
					index: State::Committed,
				}) = self.0.get(0) {
			return Some(&value)
		} else {
			None
		}
	}

	pub fn into_committed(mut self) -> Option<V> {
		self.0.truncate(1);
		if let Some(HistoricalValue {
					value,
					index: State::Committed,
				}) = self.0.pop() {
			return Some(value)
		} else {
			None
		}
	}

	/// Returns mutable latest pending historical value.
	pub fn get_mut(&mut self) -> Option<HistoricalValue<&mut V>> {
		self.0.last_mut().map(|h| h.as_mut())
	}

}
