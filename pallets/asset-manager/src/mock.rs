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
//
// The pallet-tx-pause pallet is forked from Acala's transaction-pause module https://github.com/AcalaNetwork/Acala/tree/master/modules/transaction-pause
// The original license is the following - SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

//! Mock runtime for asset-manager

use crate as pallet_asset_manager;
use frame_support::{
	construct_runtime, parameter_types,
	traits::{
		fungible::Inspect,
		fungibles::{Inspect as AssetInspect, Transfer as AssetTransfer},
		tokens::{DepositConsequence, ExistenceRequirement, WithdrawConsequence},
		ConstU32, Currency,
	},
	PalletId,
};
use frame_system as system;
use frame_system::EnsureRoot;
use manta_primitives::{
	assets::{
		AssetConfig, AssetLocation, AssetRegistrar, AssetRegistrarMetadata, AssetStorageMetadata,
		FungibleLedger, FungibleLedgerConsequence,
	},
	constants::{ASSET_MANAGER_PALLET_ID, ASSET_STRING_LIMIT},
	types::{AccountId, AssetId, Balance},
};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};
use sp_std::marker::PhantomData;
use xcm::{
	prelude::{Parachain, X1},
	v1::MultiLocation,
	VersionedMultiLocation,
};

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 78;
}

impl system::Config for Runtime {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}

parameter_types! {
	pub const AssetDeposit: Balance = 0; // Does not really matter as this will be only called by root
	pub const AssetAccountDeposit: Balance = 0;
	pub const ApprovalDeposit: Balance = 0;
	pub const AssetsStringLimit: u32 = ASSET_STRING_LIMIT;
	pub const MetadataDepositBase: Balance = 0;
	pub const MetadataDepositPerByte: Balance = 0;
}

impl pallet_assets::Config for Runtime {
	type Event = Event;
	type Balance = Balance;
	type AssetId = AssetId;
	type Currency = Balances;
	type ForceOrigin = EnsureRoot<AccountId>;
	type AssetDeposit = AssetDeposit;
	type AssetAccountDeposit = AssetAccountDeposit;
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type ApprovalDeposit = ApprovalDeposit;
	type StringLimit = AssetsStringLimit;
	type Freezer = ();
	type Extra = ();
	type WeightInfo = ();
}

parameter_types! {
	pub ExistentialDeposit: Balance = 1;
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Runtime {
	type MaxLocks = MaxLocks;
	type Balance = Balance;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
}

pub struct MantaAssetRegistrar;
use frame_support::pallet_prelude::DispatchResult;
impl AssetRegistrar<Runtime, MantaAssetConfig> for MantaAssetRegistrar {
	fn create_asset(
		asset_id: AssetId,
		min_balance: Balance,
		metadata: AssetStorageMetadata,
		is_sufficient: bool,
	) -> DispatchResult {
		Assets::force_create(
			Origin::root(),
			asset_id,
			AssetManager::account_id(),
			is_sufficient,
			min_balance,
		)?;

		Assets::force_set_metadata(
			Origin::root(),
			asset_id,
			metadata.name,
			metadata.symbol,
			metadata.decimals,
			metadata.is_frozen,
		)
	}

	fn update_asset_metadata(asset_id: AssetId, metadata: AssetStorageMetadata) -> DispatchResult {
		Assets::force_set_metadata(
			Origin::root(),
			asset_id,
			metadata.name,
			metadata.symbol,
			metadata.decimals,
			metadata.is_frozen,
		)
	}
}

pub struct MantaFungibleLedger;
impl FungibleLedger<Runtime> for MantaFungibleLedger {
	fn is_valid(asset_id: AssetId) -> Result<(), FungibleLedgerConsequence> {
		if asset_id >= <MantaAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get()
			|| asset_id == <MantaAssetConfig as AssetConfig<Runtime>>::NativeAssetId::get()
		{
			Ok(())
		} else {
			Err(FungibleLedgerConsequence::InvalidAssetId)
		}
	}

	fn can_deposit(
		asset_id: AssetId,
		account: &<Runtime as frame_system::Config>::AccountId,
		amount: Balance,
	) -> Result<(), FungibleLedgerConsequence> {
		Self::is_valid(asset_id)?;
		if asset_id == <MantaAssetConfig as AssetConfig<Runtime>>::NativeAssetId::get() {
			match Balances::can_deposit(account, amount) {
				DepositConsequence::Success => Ok(()),
				other => Err(other.into()),
			}
		} else {
			match Assets::can_deposit(asset_id, account, amount) {
				DepositConsequence::Success => Ok(()),
				other => Err(other.into()),
			}
		}
	}

	fn can_withdraw(
		asset_id: AssetId,
		account: &<Runtime as frame_system::Config>::AccountId,
		amount: Balance,
	) -> Result<(), FungibleLedgerConsequence> {
		Self::is_valid(asset_id)?;
		if asset_id == <MantaAssetConfig as AssetConfig<Runtime>>::NativeAssetId::get() {
			match Balances::can_withdraw(account, amount) {
				WithdrawConsequence::Success => Ok(()),
				other => Err(other.into()),
			}
		} else {
			match Assets::can_withdraw(asset_id, account, amount) {
				WithdrawConsequence::Success => Ok(()),
				other => Err(other.into()),
			}
		}
	}

	fn transfer(
		asset_id: AssetId,
		source: &<Runtime as frame_system::Config>::AccountId,
		dest: &<Runtime as frame_system::Config>::AccountId,
		amount: Balance,
	) -> Result<(), FungibleLedgerConsequence> {
		Self::is_valid(asset_id)?;
		if asset_id == <MantaAssetConfig as AssetConfig<Runtime>>::NativeAssetId::get() {
			<Balances as Currency<<Runtime as frame_system::Config>::AccountId>>::transfer(
				source,
				dest,
				amount,
				ExistenceRequirement::KeepAlive,
			)
			.map_err(|_| FungibleLedgerConsequence::InternalError)
		} else {
			<Assets as AssetTransfer<<Runtime as frame_system::Config>::AccountId>>::transfer(
				asset_id, source, dest, amount, true,
			)
			.map(|_| ())
			.map_err(|_| FungibleLedgerConsequence::InternalError)
		}
	}

	fn mint(
		asset_id: AssetId,
		beneficiary: &<Runtime as frame_system::Config>::AccountId,
		amount: Balance,
	) -> Result<(), FungibleLedgerConsequence> {
		Self::is_valid(asset_id)?;
		Self::can_deposit(asset_id, beneficiary, amount)?;
		if asset_id == <MantaAssetConfig as AssetConfig<Runtime>>::NativeAssetId::get() {
			let _ =
				<Balances as Currency<<Runtime as frame_system::Config>::AccountId>>::deposit_creating(
					beneficiary,
					amount,
				);
			Ok(())
		} else {
			Assets::mint(
				Origin::signed(AssetManager::account_id()),
				asset_id,
				beneficiary.clone(),
				amount,
			)
			.map(|_| ())
			.map_err(|_| FungibleLedgerConsequence::InternalError)
		}
	}
}

parameter_types! {
	pub const DummyAssetId: AssetId = 0;
	pub const NativeAssetId: AssetId = 1;
	pub const StartNonNativeAssetId: AssetId = 8;
	pub NativeAssetLocation: AssetLocation = AssetLocation(
		VersionedMultiLocation::V1(MultiLocation::new(1, X1(Parachain(1024)))));
	pub NativeAssetMetadata: AssetRegistrarMetadata = AssetRegistrarMetadata {
		name: b"Dolphin".to_vec(),
		symbol: b"DOL".to_vec(),
		decimals: 18,
		min_balance: 1u128,
		evm_address: None,
		is_frozen: false,
		is_sufficient: true,
	};
	pub const AssetManagerPalletId: PalletId = ASSET_MANAGER_PALLET_ID;
}

#[derive(Clone, Eq, PartialEq)]
pub struct MantaAssetConfig;

impl AssetConfig<Runtime> for MantaAssetConfig {
	type DummyAssetId = DummyAssetId;
	type NativeAssetId = NativeAssetId;
	type StartNonNativeAssetId = StartNonNativeAssetId;
	type AssetRegistrarMetadata = AssetRegistrarMetadata;
	type NativeAssetLocation = NativeAssetLocation;
	type NativeAssetMetadata = NativeAssetMetadata;
	type StorageMetadata = AssetStorageMetadata;
	type AssetLocation = AssetLocation;
	type AssetRegistrar = MantaAssetRegistrar;
	type FungibleLedger = MantaFungibleLedger;
}

impl pallet_asset_manager::Config for Runtime {
	type Event = Event;
	type AssetConfig = MantaAssetConfig;
	type ModifierOrigin = EnsureRoot<AccountId>;
	type PalletId = AssetManagerPalletId;
	type WeightInfo = ();
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Storage, Config, Event<T>} = 0,
		Assets: pallet_assets::{Pallet, Storage, Event<T>} = 1,
		AssetManager: pallet_asset_manager::{Pallet, Call, Storage, Event<T>} = 2,
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 3,
	}
);

pub const PALLET_BALANCES_INDEX: u8 = 3;

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default()
		.build_storage::<Runtime>()
		.unwrap();
	pallet_asset_manager::GenesisConfig::<Runtime> {
		start_id: <MantaAssetConfig as AssetConfig<Runtime>>::StartNonNativeAssetId::get(),
		_marker: PhantomData::<Runtime>::default(),
	}
	.assimilate_storage(&mut t)
	.unwrap();
	sp_io::TestExternalities::new(t)
}
