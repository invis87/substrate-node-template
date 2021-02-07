#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch,
    traits::{Currency, Get, ReservableCurrency},
};
use frame_system::ensure_signed;
use frame_system::Module;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

type BalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: frame_system::Trait {
    /// Because this pallet emits events, it depends on the runtime's definition of an event.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    /// The currency trait.
    type Currency: ReservableCurrency<Self::AccountId>;
    /// Address with threasury.
    type ThreasuryAddress: <Self as frame_system::Trait>::AccountId;
}

// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
    // A unique name is used to ensure that the pallet's storage items are isolated.
    // This name may be updated, but each pallet in the runtime must use a unique name.
    // ---------------------------------vvvvvvvvvvvvvv
    trait Store for Module<T: Trait> as EquationModule {
        Treasury get(fn no_idea): u64;
        /// The lookup table for equations.
        EquationsToSolve: map hasher(identity) u64 => Option<(T::AccountId, BalanceOf<T>)>;
        EquationsSolved get(fn u64): Option<(u64, u64)>;
    }
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
    pub enum Event<T>
    where
        Balance = BalanceOf<T>,
    {
        /// A new equation was created
        EquationCreated(u64, Balance),
        /// Equation was solved, and the given balance reward
        EquationSolved(u64, u64, u64, Balance),
    }
);

// Errors inform users that something went wrong.
decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Create equation with inappropriate number
        WrongNumber,
        /// Create equation with zero reward
        ZeroReward,
        /// Equation task already exists
        AlreadyExists,
        /// Trying to solve equation which do not exists
        NoSuchEquation,
        /// Equation already solved
        AlreadySolved,
        /// Try solve with wrong solution
        WrongSolution,
    }
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Errors must be initialized if they are used by the pallet.
        type Error = Error<T>;

        /// Address with threasury
        const ThreasuryAddress: T::AccountId  = T::ThreasuryAddress::get();

        // Events must be initialized if they are used by the pallet.
        fn equation_event() = default;

        pub fn create_problem(origin, number: u64, reward: BalanceOf<T>) -> dispatch::DispatchResult {
            ensure!(number > 1, Error::<T>::WrongNumber);
            ensure!(reward > 0, Error::<T>::ZeroReward);
            let sender = ensure_signed(origin)?;

            if let Some(_) = <EquationsToSolve<T>>::get(&number) {
                Err(Error::<T>::AlreadyExists)?
            }

            if let Some((a, b)) = EquationsSolved::get(&number) {
                Err(Error::<T>::AlreadySolved)?
            }

            T::Currency::reserve(&sender, reward.clone())?;
            Self::equation_event(RawEvent::EquationCreated(number, reward.clone()));
            <EquationsToSolve<T>>::insert(&number, (sender, reward ));
        }

        pub fn try_solve(origin, number:u64, a: u64, b: u64) -> dispatch::DispatchResult {
            // its ok to solve with 0 - if user wants to pay transaction fee for 100% loose - why
            // not?
            // ensure!(a > 0, Error::<T>::WrongNumberInSolution);
            // ensure!(b > 0, Error::<T>::WrongNumberInSolution);
            let sender = ensure_signed(origin)?;

            if let Some((a, b)) = EquationsSolved::get(&number) {
                Err(Error::<T>::AlreadySolved)?;
            }

            if let Some((asker, reward)) = <EquationsToSolve<T>>::get(&number) {
                //TODO: deal with panic in case of overflow
                if number == a*b {
                    //right solution

                    //TODO: check how to work with Balance
                    let gain = reward * 0.8;
                    let to_threasury = reward * 0.2;

                    let _ = T::Currency::unreserve(&asker, deposit.clone());
					//TODO: convert `to_threasury` to u64
                    T::Currency::transfer(&asker, &T::ThreasuryAddress::get(), to_threasury);
                    match Treasury::get() {
                        // Return an error if the value has not been set.
                        None => {
                            Treasury::put(to_threasury)
                        },
                        Some(old) => {
                            Treasury::put(to_threasury + old)
                        },
                    }

                    //send 80% to user
                    T::Currency::transfer(&asker, &sender, gain.clone());
					Self::equation_event(RawEvent::EquationSolved(number, a, b, gain));
                    <EquationsSolved<T>>::insert(number, (a, b));
                } else {
                    //wrong solution
                    Err(Error::<T>::WrongSolution)?;
                }

            } else {
                Err(Error::<T>::NoSuchEquation)?;
            }
        }
    }
}
