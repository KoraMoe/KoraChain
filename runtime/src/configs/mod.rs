// This is free and unencumbered software released into the public domain.
//
// Anyone is free to copy, modify, publish, use, compile, sell, or
// distribute this software, either in source code form or as a compiled
// binary, for any purpose, commercial or non-commercial, and by any
// means.
//
// In jurisdictions that recognize copyright laws, the author or authors
// of this software dedicate any and all copyright interest in the
// software to the public domain. We make this dedication for the benefit
// of the public at large and to the detriment of our heirs and
// successors. We intend this dedication to be an overt act of
// relinquishment in perpetuity of all present and future rights to this
// software under copyright law.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
// IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY CLAIM, DAMAGES OR
// OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
// ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
// OTHER DEALINGS IN THE SOFTWARE.
//
// For more information, please refer to <http://unlicense.org>

// Substrate and Polkadot dependencies
use alloc::vec;
use frame_support::{
	derive_impl, parameter_types,
	traits::{ConstBool, ConstU128, ConstU32, ConstU64, ConstU8, VariantCountOf},
	weights::{
		constants::{RocksDbWeight, WEIGHT_REF_TIME_PER_SECOND},
		IdentityFee, Weight,
	},
};
use codec::{Decode, Encode};
use frame_system::{
	limits::{BlockLength, BlockWeights},
	EnsureRoot, EnsureRootWithSuccess, EnsureSigned, EnsureSignedBy, EnsureWithSuccess,
};
use pallet_transaction_payment::{ConstFeeMultiplier, FungibleAdapter, Multiplier};
use sp_runtime::{
	curve::PiecewiseLinear,
	generic, impl_opaque_keys, str_array as s,
	traits::{
		self, AccountIdConversion, BlakeTwo256, Block as BlockT, Bounded, ConvertInto,
		MaybeConvert, NumberFor, OpaqueKeys, SaturatedConversion, StaticLookup, One,
	},
	transaction_validity::{TransactionPriority, TransactionSource, TransactionValidity},
	ApplyExtrinsicResult, FixedPointNumber, FixedU128, MultiSignature, MultiSigner, Perbill,
	Percent, Permill, Perquintill, RuntimeDebug,
};
use pallet_election_provider_multi_phase::{GeometricDepositBase, SolutionAccuracyOf};
use sp_version::RuntimeVersion;
use frame_support::{
	dispatch::DispatchClass,
	traits::{
		EitherOfDiverse,
		fungible::{
			NativeOrWithId
		},
		tokens::{
			imbalance::{ResolveAssetTo, ResolveTo},
			pay::{PayAssetFromAccount, PayFromAccount},
			ConversionFromAssetBalance, UnityAssetBalanceConversion
		}
	},
	weights::{
		constants::{
			BlockExecutionWeight, ExtrinsicBaseWeight
		},
	},
	PalletId
};
use frame_election_provider_support::{
	bounds::{ElectionBounds, ElectionBoundsBuilder},
	onchain, BalancingConfig, ElectionDataProvider, SequentialPhragmen, VoteWeight,
};
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sp_runtime::traits::IdentityLookup;
// Local module imports
use super::*;

const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

parameter_types! {
	pub const BlockHashCount: BlockNumber = 2400;
	pub const Version: RuntimeVersion = VERSION;

	/// We allow for 2 seconds of compute with a 6 second average block time.
	pub RuntimeBlockWeights: BlockWeights = BlockWeights::with_sensible_defaults(
		Weight::from_parts(2u64 * WEIGHT_REF_TIME_PER_SECOND, u64::MAX),
		NORMAL_DISPATCH_RATIO,
	);
	pub RuntimeBlockLength: BlockLength = BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	pub const SS58Prefix: u8 = 42;
}

/// The default types are being injected by [`derive_impl`](`frame_support::derive_impl`) from
/// [`RelayChainDefaultConfig`](`struct@frame_system::config_preludes::RelayChainDefaultConfig`),
/// but overridden as needed.
#[derive_impl(frame_system::config_preludes::RelayChainDefaultConfig)]
impl frame_system::Config for Runtime {
	/// The block type for the runtime.
	type Block = Block;
	/// Block & extrinsics weights: base values and limits.
	type BlockWeights = RuntimeBlockWeights;
	/// The maximum length of a block (in bytes).
	type BlockLength = RuntimeBlockLength;
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The type for storing how many extrinsics an account has signed.
	type Nonce = Nonce;
	/// The type for hashing blocks and tries.
	type Hash = Hash;
	/// Maximum number of block number to block hash mappings to keep (oldest pruned first).
	type BlockHashCount = BlockHashCount;
	/// The weight of database operations that the runtime can invoke.
	type DbWeight = RocksDbWeight;
	/// Version of the runtime.
	type Version = Version;
	/// The data to be stored in an account.
	type AccountData = pallet_balances::AccountData<Balance>;
	/// This is used as an identifier of the chain. 42 is the generic substrate prefix.
	type SS58Prefix = SS58Prefix;
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	// NOTE: Currently it is not possible to change the epoch duration after the chain has started.
	//       Attempting to do so will brick block production.
	pub const EpochDuration: u64 = EPOCH_DURATION_IN_SLOTS;
	pub const ExpectedBlockTime: u64 = MILLI_SECS_PER_BLOCK;
	pub const ReportLongevity: u64 =
		BondingDuration::get() as u64 * SessionsPerEra::get() as u64 * EpochDuration::get();
}

parameter_types! {
	pub const ImOnlineUnsignedPriority: TransactionPriority = TransactionPriority::MAX;
	pub const MaxAuthorities: u32 = 1000;
	pub const MaxKeys: u32 = 10_000;
	pub const MaxPeerInHeartbeats: u32 = 10_000;
}

impl pallet_babe::Config for Runtime {
	type EpochDuration = EpochDuration;
	type ExpectedBlockTime = ExpectedBlockTime;
	type EpochChangeTrigger = pallet_babe::ExternalTrigger;
	type DisabledValidators = Session;
	type WeightInfo = ();
	type MaxAuthorities = MaxAuthorities;
	type MaxNominators = MaxNominators;
	type KeyOwnerProof = sp_session::MembershipProof;
	type EquivocationReportSystem =
	pallet_babe::EquivocationReportSystem<Self, Offences, Historical, ReportLongevity>;
}

impl pallet_offences::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type IdentificationTuple = pallet_session::historical::IdentificationTuple<Self>;
	type OnOffenceHandler = Staking;
}

impl pallet_im_online::Config for Runtime {
	type AuthorityId = ImOnlineId;
	type MaxKeys = MaxKeys;
	type MaxPeerInHeartbeats = MaxPeerInHeartbeats;
	type RuntimeEvent = RuntimeEvent;
	type ValidatorSet = Historical;
	type NextSessionRotation = Babe;
	type ReportUnresponsiveness = Offences;
	type UnsignedPriority = ImOnlineUnsignedPriority;
	type WeightInfo = pallet_im_online::weights::SubstrateWeight<Runtime>;
}

impl pallet_authorship::Config for Runtime {
	type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Babe>;
	type EventHandler = (Staking, ImOnline);
}

impl pallet_session::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	type ValidatorIdOf = sp_runtime::traits::ConvertInto;
	type ShouldEndSession = Babe;
	type NextSessionRotation = Babe;
	type SessionManager = pallet_session::historical::NoteHistoricalRoot<Self, Staking>;
	type SessionHandler = <SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
	type Keys = SessionKeys;
	type DisablingStrategy = pallet_session::disabling::UpToLimitWithReEnablingDisablingStrategy;

	type WeightInfo = pallet_session::weights::SubstrateWeight<Runtime>;
}

impl pallet_session::historical::Config for Runtime {
	type FullIdentification = pallet_staking::Exposure<AccountId, Balance>;
	type FullIdentificationOf = pallet_staking::ExposureOf<Runtime>;
}

pallet_staking_reward_curve::build! {
	const REWARD_CURVE: PiecewiseLinear<'static> = curve!(
		min_inflation: 0_025_000,
		max_inflation: 0_100_000,
		ideal_stake: 0_500_000,
		falloff: 0_050_000,
		max_piece_count: 40,
		test_precision: 0_005_000,
	);
}

parameter_types! {
	pub const SessionsPerEra: sp_staking::SessionIndex = 6;
	pub const BondingDuration: sp_staking::EraIndex = 24 * 28;
	pub const SlashDeferDuration: sp_staking::EraIndex = 24 * 7; // 1/4 the bonding duration.
	pub const RewardCurve: &'static PiecewiseLinear<'static> = &REWARD_CURVE;
	pub const MaxNominators: u32 = 64;
	pub const MaxControllersInDeprecationBatch: u32 = 5900;
	pub OffchainRepeat: BlockNumber = 5;
	pub HistoryDepth: u32 = 84;
}

/// Upper limit on the number of NPOS nominations.
const MAX_QUOTA_NOMINATIONS: u32 = 16;

pub struct StakingBenchmarkingConfig;
impl pallet_staking::BenchmarkingConfig for StakingBenchmarkingConfig {
	type MaxValidators = ConstU32<1000>;
	type MaxNominators = ConstU32<5000>;
}

pub struct OnChainSeqPhragmen;
impl onchain::Config for OnChainSeqPhragmen {
	type System = Runtime;
	type Solver = SequentialPhragmen<AccountId, SolutionAccuracyOf<Runtime>>;
	type DataProvider = Staking;
	type WeightInfo = frame_election_provider_support::weights::SubstrateWeight<Runtime>;
	type MaxWinners = MaxActiveValidators;
	type Bounds = ElectionBoundsOnChain;
}

frame_election_provider_support::generate_solution_type!(
	#[compact]
	pub struct NposSolution16::<
		VoterIndex = u32,
		TargetIndex = u16,
		Accuracy = sp_runtime::PerU16,
		MaxVoters = MaxElectingVotersSolution,
	>(16)
);

/// The numbers configured here could always be more than the the maximum limits of staking pallet
/// to ensure election snapshot will not run out of memory. For now, we set them to smaller values
/// since the staking is bounded and the weight pipeline takes hours for this single pallet.
pub struct ElectionProviderBenchmarkConfig;
impl pallet_election_provider_multi_phase::BenchmarkingConfig for ElectionProviderBenchmarkConfig {
	const VOTERS: [u32; 2] = [1000, 2000];
	const TARGETS: [u32; 2] = [500, 1000];
	const ACTIVE_VOTERS: [u32; 2] = [500, 800];
	const DESIRED_TARGETS: [u32; 2] = [200, 400];
	const SNAPSHOT_MAXIMUM_VOTERS: u32 = 1000;
	const MINER_MAXIMUM_VOTERS: u32 = 1000;
	const MAXIMUM_TARGETS: u32 = 300;
}


/// Maximum number of iterations for balancing that will be executed in the embedded OCW
/// miner of election provider multi phase.
pub const MINER_MAX_ITERATIONS: u32 = 10;

/// A source of random balance for NposSolver, which is meant to be run by the OCW election miner.
pub struct OffchainRandomBalancing;

impl frame_support::pallet_prelude::Get<Option<BalancingConfig>> for OffchainRandomBalancing {
	fn get() -> Option<BalancingConfig> {
		use sp_runtime::traits::TrailingZeroInput;
		let iterations = match MINER_MAX_ITERATIONS {
			0 => 0,
			max => {
				let seed = sp_io::offchain::random_seed();
				let random = <u32>::decode(&mut TrailingZeroInput::new(&seed))
					.expect("input is padded with zeroes; qed") %
					max.saturating_add(1);
				random as usize
			},
		};

		let config = BalancingConfig { iterations, tolerance: 0 };
		Some(config)
	}
}

parameter_types! {
	// phase durations. 1/4 of the last session for each.
	pub const SignedPhase: u32 = EPOCH_DURATION_IN_BLOCKS / 4;
	pub const UnsignedPhase: u32 = EPOCH_DURATION_IN_BLOCKS / 4;

	// signed config
	pub const SignedRewardBase: Balance = 1 * UNIT;
	pub const SignedFixedDeposit: Balance = 1 * UNIT;
	pub const SignedDepositIncreaseFactor: Percent = Percent::from_percent(10);
	pub const SignedDepositByte: Balance = 1 * MILLI_UNIT;

	// miner configs
	/// We prioritize im-online heartbeats over election solution submission.
	pub const StakingUnsignedPriority: TransactionPriority = TransactionPriority::max_value() / 2;

	// miner configs
	pub const MultiPhaseUnsignedPriority: TransactionPriority = StakingUnsignedPriority::get() - 1u64;
	pub MinerMaxWeight: Weight = RuntimeBlockWeights::get()
		.get(DispatchClass::Normal)
		.max_extrinsic.expect("Normal extrinsics have a weight limit configured; qed")
		.saturating_sub(BlockExecutionWeight::get());
	// Solution can occupy 90% of normal block size
	pub MinerMaxLength: u32 = Perbill::from_rational(9u32, 10) *
		*RuntimeBlockLength::get()
		.max
		.get(DispatchClass::Normal);
}

parameter_types! {
	// Note: the EPM in this runtime runs the election on-chain. The election bounds must be
	// carefully set so that an election round fits in one block.
	pub ElectionBoundsMultiPhase: ElectionBounds = ElectionBoundsBuilder::default()
		.voters_count(10_000.into()).targets_count(1_500.into()).build();
	pub ElectionBoundsOnChain: ElectionBounds = ElectionBoundsBuilder::default()
		.voters_count(5_000.into()).targets_count(1_250.into()).build();

	pub MaxNominations: u32 = <NposSolution16 as frame_election_provider_support::NposSolution>::LIMIT as u32;
	pub MaxElectingVotersSolution: u32 = 40_000;
	// The maximum winners that can be elected by the Election pallet which is equivalent to the
	// maximum active validators the staking pallet can have.
	pub MaxActiveValidators: u32 = 1000;
}

impl pallet_election_provider_multi_phase::MinerConfig for Runtime {
	type AccountId = AccountId;
	type Solution = NposSolution16;
	type MaxVotesPerVoter =
	<<Self as pallet_election_provider_multi_phase::Config>::DataProvider as ElectionDataProvider>::MaxVotesPerVoter;
	type MaxLength = MinerMaxLength;
	type MaxWeight = MinerMaxWeight;
	type MaxWinners = MaxActiveValidators;
	// The unsigned submissions have to respect the weight of the submit_unsigned call, thus their
	// weight estimate function is wired to this call's weight.
	fn solution_weight(v: u32, t: u32, a: u32, d: u32) -> Weight {
		<
		<Self as pallet_election_provider_multi_phase::Config>::WeightInfo
		as
		pallet_election_provider_multi_phase::WeightInfo
		>::submit_unsigned(v, t, a, d)
	}
}

impl pallet_election_provider_multi_phase::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type EstimateCallFee = TransactionPayment;
	type UnsignedPhase = UnsignedPhase;
	type SignedPhase = SignedPhase;
	type BetterSignedThreshold = ();
	type OffchainRepeat = OffchainRepeat;
	type MinerTxPriority = MultiPhaseUnsignedPriority;
	type MinerConfig = Self;
	type SignedMaxSubmissions = ConstU32<10>;
	type SignedMaxWeight = MinerMaxWeight;
	type SignedMaxRefunds = ConstU32<3>;
	type SignedRewardBase = SignedRewardBase;
	type SignedDepositByte = SignedDepositByte;
	type SignedDepositWeight = ();
	// EnsureRootOrHalfCouncil
	type MaxWinners = MaxActiveValidators;
	type SignedDepositBase =
	GeometricDepositBase<Balance, SignedFixedDeposit, SignedDepositIncreaseFactor>;
	type ElectionBounds = ElectionBoundsMultiPhase;
	type SlashHandler = ();
	// burn slashes
	type RewardHandler = ();
	// rewards are minted from the void
	type DataProvider = Staking;
	type Fallback = onchain::OnChainExecution<OnChainSeqPhragmen>;
	type GovernanceFallback = onchain::OnChainExecution<OnChainSeqPhragmen>;
	type Solver = SequentialPhragmen<AccountId, SolutionAccuracyOf<Self>, OffchainRandomBalancing>;
	type ForceOrigin = EnsureRoot<AccountId>;
	type BenchmarkingConfig = ElectionProviderBenchmarkConfig;
	type WeightInfo = pallet_election_provider_multi_phase::weights::SubstrateWeight<Self>;
}

parameter_types! {
	pub TreasuryAccount: AccountId = Treasury::account_id();
}

impl pallet_staking::Config for Runtime {
	type OldCurrency = Balances;
	type Currency = Balances;
	type RuntimeHoldReason = RuntimeHoldReason;
	type CurrencyBalance = Balance;
	type UnixTime = Timestamp;
	type CurrencyToVote = sp_staking::currency_to_vote::U128CurrencyToVote;
	type ElectionProvider = ElectionProviderMultiPhase;
	type GenesisElectionProvider = onchain::OnChainExecution<OnChainSeqPhragmen>;
	type NominationsQuota = pallet_staking::FixedNominationsQuota<MAX_QUOTA_NOMINATIONS>;
	type HistoryDepth = HistoryDepth;
	type RewardRemainder = ResolveTo<TreasuryAccount, Balances>;
	type RuntimeEvent = RuntimeEvent;
	type Slash = ResolveTo<TreasuryAccount, Balances>;
	// send the slashed funds to the treasury.
	type Reward = ();
	// rewards are minted from the void
	type SessionsPerEra = SessionsPerEra;
	type BondingDuration = BondingDuration;
	type SlashDeferDuration = SlashDeferDuration;
	/// A super-majority of the council can cancel the slash.
	type AdminOrigin = EnsureRoot<AccountId>;
	type SessionInterface = Self;
	type EraPayout = pallet_staking::ConvertCurve<RewardCurve>;
	type NextNewSession = Session;
	type MaxExposurePageSize = ConstU32<256>;
	type VoterList = VoterList;
	// This a placeholder, to be introduced in the next PR as an instance of bags-list
	type TargetList = pallet_staking::UseValidatorsMap<Self>;
	type MaxUnlockingChunks = ConstU32<32>;
	type MaxControllersInDeprecationBatch = MaxControllersInDeprecationBatch;
	type EventListeners = ();
	type Filter = ();
	type BenchmarkingConfig = StakingBenchmarkingConfig;
	type WeightInfo = pallet_staking::weights::SubstrateWeight<Runtime>;
}

impl pallet_utility::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = pallet_utility::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const SpendPeriod: BlockNumber = 1 * DAYS;
	pub const Burn: Permill = Permill::from_percent(50);
	pub const TipCountdown: BlockNumber = 1 * DAYS;
	pub const TipFindersFee: Percent = Percent::from_percent(20);
	pub const TipReportDepositBase: Balance = 1 * UNIT;
	pub const DataDepositPerByte: Balance = 1 * MILLI_UNIT;
	pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
	pub const MaximumReasonLength: u32 = 300;
	pub const MaxApprovals: u32 = 100;
	pub const MaxBalance: Balance = Balance::MAX;
	pub const SpendPayoutPeriod: BlockNumber = 30 * DAYS;
}

impl pallet_treasury::Config for Runtime {
	type Currency = Balances;
	type RejectOrigin = EnsureRoot<AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type SpendPeriod = SpendPeriod;
	type Burn = Burn;
	type PalletId = TreasuryPalletId;
	type BurnDestination = ();
	type WeightInfo = pallet_treasury::weights::SubstrateWeight<Runtime>;
	type SpendFunds = ();
	type MaxApprovals = MaxApprovals;
	type SpendOrigin = EnsureWithSuccess<EnsureRoot<AccountId>, AccountId, MaxBalance>;
	// Simplified configuration to avoid trait implementation issues
	type AssetKind = ();
	type Beneficiary = AccountId;
	type BeneficiaryLookup = IdentityLookup<Self::Beneficiary>;
	type Paymaster = PayFromAccount<Balances, TreasuryAccount>;
	type BalanceConverter = UnityAssetBalanceConversion;
	type PayoutPeriod = SpendPayoutPeriod;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
	type BlockNumberProvider = System;
}

impl pallet_grandpa::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;

	type WeightInfo = ();
	type MaxAuthorities = ConstU32<32>;
	type MaxNominators = ConstU32<0>;
	type MaxSetIdSessionEntries = ConstU64<0>;

	type KeyOwnerProof = sp_core::Void;
	type EquivocationReportSystem = ();
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = Babe;
	type MinimumPeriod = ConstU64<{ SLOT_DURATION / 2 }>;
	type WeightInfo = ();
}

impl pallet_balances::Config for Runtime {
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	type RuntimeHoldReason = RuntimeHoldReason;
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
	/// The type for recording an account's balance.
	type Balance = Balance;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU128<EXISTENTIAL_DEPOSIT>;
	type AccountStore = System;
	type ReserveIdentifier = [u8; 8];
	type FreezeIdentifier = RuntimeFreezeReason;
	type MaxLocks = ConstU32<50>;
	type MaxReserves = ();
	type MaxFreezes = VariantCountOf<RuntimeFreezeReason>;
	type DoneSlashHandler = ();
}

parameter_types! {
	pub const BagThresholds: &'static [u64] = &VOTER_BAG_THRESHOLDS;
}

type VoterBagsListInstance = pallet_bags_list::Instance1;
impl pallet_bags_list::Config<VoterBagsListInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_bags_list::weights::SubstrateWeight<Runtime>;
	/// The voter bags-list is loosely kept up to date, and the real source of truth for the score
	/// of each node is the staking pallet.
	type ScoreProvider = Staking;
	type BagThresholds = BagThresholds;
	type Score = VoteWeight;
}

parameter_types! {
	pub FeeMultiplier: Multiplier = Multiplier::one();
}

impl pallet_transaction_payment::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnChargeTransaction = FungibleAdapter<Balances, ()>;
	type WeightToFee = IdentityFee<Balance>;
	type LengthToFee = IdentityFee<Balance>;
	type FeeMultiplierUpdate = ConstFeeMultiplier<FeeMultiplier>;
	type OperationalFeeMultiplier = ConstU8<5>;
	type WeightInfo = pallet_transaction_payment::weights::SubstrateWeight<Runtime>;
}

impl pallet_sudo::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type WeightInfo = pallet_sudo::weights::SubstrateWeight<Runtime>;
}

/// Configure the pallet-template in pallets/template.
impl pallet_template::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_template::weights::SubstrateWeight<Runtime>;
}

impl<LocalCall> frame_system::offchain::CreateInherent<LocalCall> for Runtime
where
	RuntimeCall: From<LocalCall>,
{
	fn create_inherent(call: RuntimeCall) -> UncheckedExtrinsic {
		generic::UncheckedExtrinsic::new_bare(call).into()
	}
}

impl<C> frame_system::offchain::CreateTransactionBase<C> for Runtime
where
	RuntimeCall: From<C>,
{
	type Extrinsic = UncheckedExtrinsic;
	type RuntimeCall = RuntimeCall;
}

/// Upper thresholds delimiting the bag list.
pub const VOTER_BAG_THRESHOLDS: [u64; 200] = [
	100_000_000_000_000,
	106_282_535_907_434,
	112_959_774_389_150,
	120_056_512_776_105,
	127_599_106_300_477,
	135_615_565_971_369,
	144_135_662_599_590,
	153_191_037_357_827,
	162_815_319_286_803,
	173_044_250_183_800,
	183_915_817_337_347,
	195_470_394_601_017,
	207_750_892_330_229,
	220_802_916_738_890,
	234_674_939_267_673,
	249_418_476_592_914,
	265_088_281_944_639,
	281_742_548_444_211,
	299_443_125_216_738,
	318_255_747_080_822,
	338_250_278_668_647,
	359_500_973_883_001,
	382_086_751_654_776,
	406_091_489_025_036,
	431_604_332_640_068,
	458_720_029_816_222,
	487_539_280_404_019,
	518_169_110_758_247,
	550_723_271_202_866,
	585_322_658_466_782,
	622_095_764_659_305,
	661_179_154_452_653,
	702_717_972_243_610,
	746_866_481_177_808,
	793_788_636_038_393,
	843_658_692_126_636,
	896_661_852_395_681,
	952_994_955_240_703,
	1_012_867_205_499_736,
	1_076_500_951_379_881,
	1_144_132_510_194_192,
	1_216_013_045_975_769,
	1_292_409_502_228_280,
	1_373_605_593_276_862,
	1_459_902_857_901_004,
	1_551_621_779_162_291,
	1_649_102_974_585_730,
	1_752_708_461_114_642,
	1_862_822_999_536_805,
	1_979_855_523_374_646,
	2_104_240_657_545_975,
	2_236_440_332_435_128,
	2_376_945_499_368_703,
	2_526_277_953_866_680,
	2_684_992_273_439_945,
	2_853_677_877_130_641,
	3_032_961_214_443_876,
	3_223_508_091_799_862,
	3_426_026_145_146_232,
	3_641_267_467_913_124,
	3_870_031_404_070_482,
	4_113_167_516_660_186,
	4_371_578_742_827_277,
	4_646_224_747_067_156,
	4_938_125_485_141_739,
	5_248_364_991_899_922,
	5_578_095_407_069_235,
	5_928_541_253_969_291,
	6_301_003_987_036_955,
	6_696_866_825_051_405,
	7_117_599_888_008_300,
	7_564_765_656_719_910,
	8_040_024_775_416_580,
	8_545_142_218_898_723,
	9_081_993_847_142_344,
	9_652_573_371_700_016,
	10_258_999_759_768_490,
	10_903_525_103_419_522,
	11_588_542_983_217_942,
	12_316_597_357_287_042,
	13_090_392_008_832_678,
	13_912_800_587_211_472,
	14_786_877_279_832_732,
	15_715_868_154_526_436,
	16_703_223_214_499_558,
	17_752_609_210_649_358,
	18_867_923_258_814_856,
	20_053_307_312_537_008,
	21_313_163_545_075_252,
	22_652_170_697_804_756,
	24_075_301_455_707_600,
	25_587_840_914_485_432,
	27_195_406_207_875_088,
	28_903_967_368_057_400,
	30_719_869_496_628_636,
	32_649_856_328_471_220,
	34_701_095_276_033_064,
	36_881_204_047_022_752,
	39_198_278_934_370_992,
	41_660_924_883_519_016,
	44_278_287_448_695_240,
	47_060_086_756_856_400,
	50_016_653_605_425_536,
	53_158_967_827_883_320,
	56_498_699_069_691_424,
	60_048_250_125_977_912,
	63_820_803_001_928_304,
	67_830_367_866_937_216,
	72_091_835_084_322_176,
	76_621_030_509_822_880,
	81_434_774_264_248_528,
	86_550_943_198_537_824,
	91_988_537_283_208_848,
	97_767_750_168_749_840,
	103_910_044_178_992_000,
	110_438_230_015_967_792,
	117_376_551_472_255_616,
	124_750_775_465_407_920,
	132_588_287_728_824_640,
	140_918_194_514_440_064,
	149_771_430_684_917_568,
	159_180_874_596_775_264,
	169_181_470_201_085_280,
	179_810_356_815_193_344,
	191_107_007_047_393_216,
	203_113_373_386_768_288,
	215_874_044_002_592_672,
	229_436_408_331_885_600,
	243_850_833_070_063_392,
	259_170_849_218_267_264,
	275_453_350_882_006_752,
	292_758_806_559_399_232,
	311_151_483_703_668_992,
	330_699_687_393_865_920,
	351_476_014_000_157_824,
	373_557_620_785_735_808,
	397_026_512_446_556_096,
	421_969_845_653_044_224,
	448_480_252_724_740_928,
	476_656_185_639_923_904,
	506_602_281_657_757_760,
	538_429_751_910_786_752,
	572_256_794_410_890_176,
	608_209_033_002_485_632,
	646_419_983_893_124_352,
	687_031_551_494_039_552,
	730_194_555_412_054_016,
	776_069_290_549_944_960,
	824_826_122_395_314_176,
	876_646_119_708_695_936,
	931_721_726_960_522_368,
	990_257_479_014_182_144,
	1_052_470_760_709_299_712,
	1_118_592_614_166_106_112,
	1_188_868_596_808_997_376,
	1_263_559_693_295_730_432,
	1_342_943_284_738_898_688,
	1_427_314_178_819_094_784,
	1_516_985_704_615_302_400,
	1_612_290_876_218_400_768,
	1_713_583_629_449_105_408,
	1_821_240_136_273_157_632,
	1_935_660_201_795_120_128,
	2_057_268_749_018_809_600,
	2_186_517_396_888_336_384,
	2_323_886_137_470_138_880,
	2_469_885_118_504_583_168,
	2_625_056_537_947_004_416,
	2_789_976_657_533_970_944,
	2_965_257_942_852_572_160,
	3_151_551_337_860_326_400,
	3_349_548_682_302_620_672,
	3_559_985_281_005_267_968,
	3_783_642_634_583_792_128,
	4_021_351_341_710_503_936,
	4_273_994_183_717_548_544,
	4_542_509_402_991_247_872,
	4_827_894_187_332_742_144,
	5_131_208_373_224_844_288,
	5_453_578_381_757_959_168,
	5_796_201_401_831_965_696,
	6_160_349_836_169_256_960,
	6_547_376_026_650_146_816,
	6_958_717_276_519_173_120,
	7_395_901_188_113_309_696,
	7_860_551_335_934_872_576,
	8_354_393_296_137_270_272,
	8_879_261_054_815_360_000,
	9_437_103_818_898_946_048,
	10_029_993_254_943_105_024,
	10_660_131_182_698_121_216,
	11_329_857_752_030_707_712,
	12_041_660_133_563_240_448,
	12_798_181_755_305_525_248,
	13_602_232_119_581_272_064,
	14_456_797_236_706_498_560,
	15_365_050_714_167_523_328,
	16_330_365_542_480_556_032,
	17_356_326_621_502_140_416,
	18_446_744_073_709_551_615,
];