// Copyright 2020-2022 Manta Network.
// This file is part of Manta.
//
// Manta is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Manta is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Manta.  If not, see <http://www.gnu.org/licenses/>.

use crate::{
	fee::{
		FEES_PERCENTAGE_TO_AUTHOR, FEES_PERCENTAGE_TO_TREASURY, TIPS_PERCENTAGE_TO_AUTHOR,
		TIPS_PERCENTAGE_TO_TREASURY,
	},
	Authorship, Balances, NegativeImbalance, Treasury,
};
use frame_support::traits::{Currency, Imbalance, OnUnbalanced};

pub struct Author;
impl OnUnbalanced<NegativeImbalance> for Author {
	fn on_nonzero_unbalanced(amount: NegativeImbalance) {
		if let Some(author) = Authorship::author() {
			Balances::resolve_creating(&author, amount);
		}
	}
}

pub struct DealWithFees;
impl OnUnbalanced<NegativeImbalance> for DealWithFees {
	fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item = NegativeImbalance>) {
		if let Some(fees) = fees_then_tips.next() {
			// for fees, 40% to treasury, 60% to author
			let mut split = fees.ration(
				FEES_PERCENTAGE_TO_TREASURY.into(),
				FEES_PERCENTAGE_TO_AUTHOR.into(),
			);
			if let Some(tips) = fees_then_tips.next() {
				// for tips, 100% to block author.
				tips.ration_merge_into(
					TIPS_PERCENTAGE_TO_TREASURY.into(),
					TIPS_PERCENTAGE_TO_AUTHOR.into(),
					&mut split,
				);
			}

			Treasury::on_unbalanced(split.0);
			Author::on_unbalanced(split.1);
		}
	}
}
