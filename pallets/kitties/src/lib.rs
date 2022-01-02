#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

mod mock;
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_support::{
		// sp_runtime::traits::Hash,
		sp_runtime::traits::{AtLeast32BitUnsigned, Bounded, CheckedAdd, One},
		traits::{ExistenceRequirement, Currency, Randomness, ReservableCurrency},
		transactional,
	};
	use frame_system::pallet_prelude::*;
	use scale_info::TypeInfo;
	use sp_io::hashing::blake2_128;

	#[cfg(feature = "std")]
	use frame_support::serde::{Deserialize, Serialize};

	type AccountOf<T> = <T as frame_system::Config>::AccountId;
	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	// Struct for holding Kitty information.
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Kitty<T: Config> {
		pub dna: [u8; 16], // Using 16 bytes to represent a kitty DNA
		pub price: Option<BalanceOf<T>>,
		pub gender: Gender,
		pub owner: AccountOf<T>,
	}

	// Enum declaration for Gender.
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub enum Gender {
		Male,
		Female,
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types it depends on.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The Currency handler for the Kitties pallet.
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

		/// The type of Randomness we want to specify for pallet.
		type KittyRandomness: Randomness<Self::Hash, Self::BlockNumber>;

		/// The maximum amount of Kitties a single account can own.
		#[pallet::constant]
		type MaxKittyOwned: Get<u32>;

		/// The type of Kitty ID
		type KittyIndex: Parameter + AtLeast32BitUnsigned + Default + Copy + Bounded + CheckedAdd;

		/// The staking balance when create_kitty
		#[pallet::constant]
		type ReserveForCreateKitty: Get<BalanceOf<Self>>;
	}

	// Errors.
	#[pallet::error]
	pub enum Error<T> {
		/// Handles arithmetic overflow when incrementing the Kitty counter.
		KittyCntOverflow,
		/// An account cannot own more Kitties than `MaxKittyCount`.
		ExceedMaxKittyOwned,
		/// Buyer cannot be the owner.
		BuyerIsKittyOwner,
		/// Cannot transfer a kitty to its owner.
		TransferToSelf,
		/// Handles checking whether the Kitty exists.
		KittyNotExist,
		/// Handles checking that the Kitty is owned by the account transferring, buying or setting a price for it.
		NotKittyOwner,
		/// Ensures the Kitty is for sale.
		KittyNotForSale,
		/// Ensures that the buying price is greater than the asking price.
		KittyBidPriceTooLow,
		/// Ensures that an account has enough funds to purchase a Kitty.
		NotEnoughBalance,
		/// Ensures that breed kitties not the same.
		BreedSameParent,
		/// Ensures that the kitty index is valid.
		InvalidKittyIndex,
		/// Ensures that your free balance is enough.
		InvalidReserveAmount,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new Kitty was successfully created. \[sender, kitty_id\]
		Created(T::AccountId, T::KittyIndex),
		/// Kitty price was successfully set. \[sender, kitty_id, new_price\]
		PriceSet(T::AccountId, T::KittyIndex, Option<BalanceOf<T>>),
		/// A Kitty was successfully transferred. \[from, to, kitty_id\]
		Transferred(T::AccountId, T::AccountId, T::KittyIndex),
		/// A Kitty was successfully bought. \[buyer, seller, kitty_id, bid_price\]
		Bought(T::AccountId, T::AccountId, T::KittyIndex, BalanceOf<T>),
	}

	/// Keeps track of the number of Kitties in existence.
	#[pallet::storage]
	#[pallet::getter(fn kitty_cnt)]
	pub(super) type KittyCnt<T: Config> = StorageValue<_, T::KittyIndex>;

	/// Stores a Kitty's unique traits, owner and price.
	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub(super) type Kitties<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, Kitty<T>>;

	/// Keeps track of what accounts own what Kitties.
	#[pallet::storage]
	#[pallet::getter(fn kitties_owned)]
	pub(super) type KittiesOwned<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		BoundedVec<T::KittyIndex, T::MaxKittyOwned>,
		ValueQuery,
	>;

	// Our pallet's genesis configuration.
	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub kitties: Vec<(T::AccountId, [u8; 16], Gender)>,
	}

	// Required to implement default for GenesisConfig.
	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> GenesisConfig<T> {
			GenesisConfig { kitties: vec![] }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			// When building a kitty from genesis config, we require the dna and gender to be supplied.
			for (acct, dna, gender) in &self.kitties {
				let _ = <Pallet<T>>::mint(acct, Some(*dna), Some(gender.clone()));
			}
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new unique kitty.
		///
		/// The actual kitty creation is done in the `mint()` function.
		#[pallet::weight(100)]
		#[transactional]
		pub fn create_kitty(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			T::Currency::reserve(&sender, T::ReserveForCreateKitty::get()).map_err(|_| Error::<T>::InvalidReserveAmount)?;
			let kitty_id = Self::mint(&sender, None, None)?;
			// Logging to the console
			log::info!("A kitty is born with ID: {:?}.", kitty_id);

			Self::deposit_event(Event::Created(sender, kitty_id));
			Ok(())
		}

		/// Set the price for a Kitty.
		///
		/// Updates Kitty price and updates storage.
		#[pallet::weight(100)]
		pub fn set_price(
			origin: OriginFor<T>,
			kitty_id: T::KittyIndex,
			new_price: Option<BalanceOf<T>>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			ensure!(Self::is_kitty_owner(&kitty_id, &sender)?, <Error<T>>::NotKittyOwner);

			let mut kitty = Self::kitties(&kitty_id).ok_or(<Error<T>>::KittyNotExist)?;

			kitty.price = new_price;
			<Kitties<T>>::insert(&kitty_id, kitty);

			Self::deposit_event(Event::PriceSet(sender, kitty_id, new_price));

			Ok(())
		}

		/// Directly transfer a kitty to another recipient.
		///
		/// Any account that holds a kitty can send it to another Account. This will reset the asking
		/// price of the kitty, marking it not for sale.
		#[pallet::weight(100)]
		pub fn transfer(
			origin: OriginFor<T>,
			to: T::AccountId,
			kitty_id: T::KittyIndex,
		) -> DispatchResult {
			let from = ensure_signed(origin)?;

			// Ensure the kitty exists and is called by the kitty owner
			ensure!(Self::is_kitty_owner(&kitty_id, &from)?, <Error<T>>::NotKittyOwner);

			// Verify the kitty is not transferring back to its owner.
			ensure!(from != to, <Error<T>>::TransferToSelf);

			// Verify the recipient has the capacity to receive one more kitty
			let to_owned = <KittiesOwned<T>>::get(&to);
			ensure!(
				(to_owned.len() as u32) < T::MaxKittyOwned::get(),
				<Error<T>>::ExceedMaxKittyOwned
			);

			Self::transfer_kitty_to(&kitty_id, &to)?;

			Self::deposit_event(Event::Transferred(from, to, kitty_id));

			Ok(())
		}

		/// Buy a saleable Kitty. The bid price provided from the buyer has to be equal or higher
		/// than the ask price from the seller.
		///
		/// This will reset the asking price of the kitty, marking it not for sale.
		/// Marking this method `transactional` so when an error is returned, we ensure no storage is changed.
		#[transactional]
		#[pallet::weight(100)]
		pub fn buy_kitty(
			origin: OriginFor<T>,
			kitty_id: T::KittyIndex,
			bid_price: BalanceOf<T>,
		) -> DispatchResult {
			let buyer = ensure_signed(origin)?;

			// Check the kitty exists and buyer is not the current kitty owner
			let kitty = Self::kitties(&kitty_id).ok_or(<Error<T>>::KittyNotExist)?;
			ensure!(kitty.owner != buyer, <Error<T>>::BuyerIsKittyOwner);

			// Check the kitty is for sale and the kitty ask price <= bid_price
			if let Some(ask_price) = kitty.price {
				ensure!(ask_price <= bid_price, <Error<T>>::KittyBidPriceTooLow);
			} else {
				return Err(<Error<T>>::KittyNotForSale.into());
			}

			// Check the buyer has enough free balance
			ensure!(T::Currency::free_balance(&buyer) >= bid_price, <Error<T>>::NotEnoughBalance);

			let to_owned = <KittiesOwned<T>>::get(&buyer);
			ensure!(
				(to_owned.len() as u32) < T::MaxKittyOwned::get(),
				<Error<T>>::ExceedMaxKittyOwned
			);

			let seller = kitty.owner;

			// Transfer the amount from buyer to seller
			T::Currency::transfer(&buyer, &seller, bid_price, ExistenceRequirement::KeepAlive)?;

			// Transfer the kitty from seller to buyer
			Self::transfer_kitty_to(&kitty_id, &buyer)?;

			// Deposit relevant Event
			Self::deposit_event(Event::Bought(buyer, seller, kitty_id, bid_price));

			Ok(())
		}

		/// Breed a Kitty.
		///
		/// Breed two kitties to create a new generation
		/// of Kitties.
		#[pallet::weight(100)]
		pub fn breed_kitty(
			origin: OriginFor<T>,
			parent1: T::KittyIndex,
			parent2: T::KittyIndex,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			ensure!(parent1 != parent2, Error::<T>::BreedSameParent);

			// Check: Verify `sender` owns both kitties (and both kitties exist).
			ensure!(Self::is_kitty_owner(&parent1, &sender)?, <Error<T>>::NotKittyOwner);
			ensure!(Self::is_kitty_owner(&parent2, &sender)?, <Error<T>>::NotKittyOwner);

			// Breed two Kitties using unique DNA
			let new_dna = Self::breed_dna(&parent1, &parent2)?;

			// Mint new Kitty using new DNA
			Self::mint(&sender, Some(new_dna), None)?;

			Ok(())
		}
	}

	//** Our helper functions.**//
	impl<T: Config> Pallet<T> {
		// helper function for Kitty struct
		fn gen_gender() -> Gender {
			let random = T::KittyRandomness::random(&b"gender"[..]).0;
			match random.as_ref()[0] % 2 {
				0 => Gender::Male,
				_ => Gender::Female,
			}
		}

		// helper funtion to randomly generate DNA
		fn gen_dna() -> [u8; 16] {
			let payload = (
				T::KittyRandomness::random(&b"dna"[..]).0,
				<frame_system::Pallet<T>>::block_number(),
			);
			payload.using_encoded(blake2_128)
		}

		// Create new DNA with existing DNA
		pub fn breed_dna(parent1: &T::KittyIndex, parent2: &T::KittyIndex) -> Result<[u8; 16], Error<T>> {
			let dna1 = Self::kitties(parent1).ok_or(<Error<T>>::KittyNotExist)?.dna;
			let dna2 = Self::kitties(parent2).ok_or(<Error<T>>::KittyNotExist)?.dna;

			let mut new_dna = Self::gen_dna();
			for i in 0..new_dna.len() {
				new_dna[i] = (new_dna[i] & dna1[i]) | (!new_dna[i] & dna2[i]);
			}
			Ok(new_dna)
		}

		// Helper to mint a Kitty.
		pub fn mint(
			owner: &T::AccountId,
			dna: Option<[u8; 16]>,
			gender: Option<Gender>,
		) -> Result<T::KittyIndex, Error<T>> {
			let kitty = Kitty::<T> {
				dna: dna.unwrap_or_else(Self::gen_dna),
				price: None,
				gender: gender.unwrap_or_else(Self::gen_gender),
				owner: owner.clone(),
			};

			// let kitty_id = T::Hashing::hash_of(&kitty);

			// Performs this operation first as it may fail
			// let new_cnt = match Self::kitty_cnt() {
			// 	Some(cnt) => {
			// 		ensure!(cnt < T::KittyIndex::max_value(), <Error<T>>::KittyCntOverflow);
			// 		cnt + 1u32.into()
			// 	},
			// 	None => 1u32.into(),
			// };
			let new_cnt = match Self::kitty_cnt() {
				Some(cnt) => {
					cnt.checked_add(&One::one()).ok_or(<Error<T>>::KittyCntOverflow)?
				},
				None => One::one(),
			};
			// let new_cnt = Self::kitty_cnt().unwrap().checked_add(&One::one()).ok_or(<Error<T>>::KittyCntOverflow)?;

			let kitty_id = new_cnt;

			// Performs this operation first as it may fail
			<KittiesOwned<T>>::try_mutate(&owner, |kitty_vec| kitty_vec.try_push(kitty_id))
				.map_err(|_| <Error<T>>::ExceedMaxKittyOwned)?;

			<Kitties<T>>::insert(kitty_id, kitty);
			<KittyCnt<T>>::put(new_cnt);

			Ok(kitty_id)
		}

		// Helper to check correct kitty owner
		pub fn is_kitty_owner(kitty_id: &T::KittyIndex, acct: &T::AccountId) -> Result<bool, Error<T>> {
			match Self::kitties(kitty_id) {
				Some(kitty) => Ok(kitty.owner == *acct),
				None => Err(<Error<T>>::KittyNotExist),
			}
		}

		#[transactional]
		pub fn transfer_kitty_to(kitty_id: &T::KittyIndex, to: &T::AccountId) -> Result<(), Error<T>> {
			let mut kitty = Self::kitties(&kitty_id).ok_or(<Error<T>>::KittyNotExist)?;

			let prev_owner = kitty.owner.clone();

			// Remove `kitty_id` from the KittyOwned vector of `prev_kitty_owner`
			<KittiesOwned<T>>::try_mutate(&prev_owner, |owned| {
				if let Some(ind) = owned.iter().position(|&id| id == *kitty_id) {
					owned.swap_remove(ind);
					return Ok(());
				}
				Err(())
			})
			.map_err(|_| <Error<T>>::KittyNotExist)?;

			// Update the kitty owner
			kitty.owner = to.clone();
			// Reset the ask price so the kitty is not for sale until `set_price()` is called
			// by the current owner.
			kitty.price = None;

			<Kitties<T>>::insert(kitty_id, kitty);

			<KittiesOwned<T>>::try_mutate(to, |vec| vec.try_push(*kitty_id))
				.map_err(|_| <Error<T>>::ExceedMaxKittyOwned)?;

			Ok(())
		}
	}
}
