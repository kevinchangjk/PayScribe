use super::redis::{Debt, UserBalance};
use std::cmp::Ordering;

/* Optimizer is purely for simplifying the debts of a group.
 * It will take in current balances of users in a group chat,
 * simplify them with a greedy algorithm, and return the debts owed.
 */

/* Utility Functions */

// Custom comparison function, only to compare the balance amount.
fn compare(a: &UserBalance, b: &UserBalance) -> Ordering {
    if a.balance < b.balance {
        Ordering::Less
    } else if a.balance > b.balance {
        Ordering::Greater
    } else {
        Ordering::Equal
    }
}

// Sorts balances in ascending order, from largest debtor, to largest creditor.
fn sort_balances(balances: &mut Vec<UserBalance>) -> () {
    balances.sort_by(compare);
}

/* Main function of Optimizer.
* Takes in a vector of balances and returns a vector of debts.
* Important: implicitly assumed that all balances sum up to 0.
*/
pub fn optimize_debts(balances: Vec<UserBalance>) -> Vec<Debt> {
    let mut sorted_balances = balances.clone();
    sort_balances(&mut sorted_balances);

    let mut debts: Vec<Debt> = Vec::new();
    let mut left = 0;
    let mut right = sorted_balances.len() - 1;

    while right > left {
        // Get the minimum of the amounts
        let amount = sorted_balances[left]
            .balance
            .abs()
            .min(sorted_balances[right].balance);

        // Add debt to the list
        if amount > 0 {
            let debt = Debt {
                debtor: sorted_balances[left].username.clone(),
                creditor: sorted_balances[right].username.clone(),
                currency: sorted_balances[left].currency.clone(),
                amount,
            };
            debts.push(debt);
        }

        // If debtor pays in full, move left pointer
        // If creditor is fully paid, move right pointer
        // If both, move both pointers
        // Else, update the amounts remaining
        if amount == sorted_balances[left].balance.abs() {
            left += 1;
        } else {
            sorted_balances[left].balance = amount + sorted_balances[left].balance;
        }
        if amount == sorted_balances[right].balance {
            right -= 1;
        } else {
            sorted_balances[right].balance -= amount;
        }
    }

    debts
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    // Utility function to check if the optimized solution is correct
    fn is_solution_correct(balances: Vec<UserBalance>, debts: Vec<Debt>) -> bool {
        let mut resulting_balances: HashMap<String, i64> = HashMap::new();
        for balance in balances {
            resulting_balances.insert(balance.username, balance.balance);
        }

        for debt in debts {
            // Owed amount cannot be negative
            if debt.amount < 0 {
                return false;
            }

            let new_debtor_balance =
                resulting_balances.get(&debt.debtor).unwrap_or(&0) + debt.amount;
            let new_creditor_balance =
                resulting_balances.get(&debt.creditor).unwrap_or(&0) - debt.amount;
            resulting_balances.insert(debt.debtor, new_debtor_balance);
            resulting_balances.insert(debt.creditor, new_creditor_balance);
        }

        for (_, balance) in resulting_balances {
            if balance != 0 {
                return false;
            }
        }

        true
    }

    #[test]
    fn test_compare() {
        let balances = vec![
            UserBalance {
                username: "user1".to_string(),
                currency: "USD".to_string(),
                balance: 1000,
            },
            UserBalance {
                username: "user2".to_string(),
                currency: "USD".to_string(),
                balance: -1000,
            },
        ];

        assert_eq!(compare(&balances[0], &balances[1]), Ordering::Greater);
    }

    #[test]
    fn test_sort_balances() {
        let mut balances = vec![
            UserBalance {
                username: "user1".to_string(),
                currency: "USD".to_string(),
                balance: 1000,
            },
            UserBalance {
                username: "user2".to_string(),
                currency: "USD".to_string(),
                balance: -1000,
            },
            UserBalance {
                username: "user3".to_string(),
                currency: "USD".to_string(),
                balance: 0,
            },
            UserBalance {
                username: "user4".to_string(),
                currency: "USD".to_string(),
                balance: 500,
            },
            UserBalance {
                username: "user5".to_string(),
                currency: "USD".to_string(),
                balance: -500,
            },
        ];

        // Sort balances
        sort_balances(&mut balances);

        // Check order of balances
        assert_eq!(
            balances,
            vec![
                UserBalance {
                    username: "user2".to_string(),
                    currency: "USD".to_string(),
                    balance: -1000,
                },
                UserBalance {
                    username: "user5".to_string(),
                    currency: "USD".to_string(),
                    balance: -500,
                },
                UserBalance {
                    username: "user3".to_string(),
                    currency: "USD".to_string(),
                    balance: 0,
                },
                UserBalance {
                    username: "user4".to_string(),
                    currency: "USD".to_string(),
                    balance: 500,
                },
                UserBalance {
                    username: "user1".to_string(),
                    currency: "USD".to_string(),
                    balance: 1000,
                },
            ]
        );
    }

    #[test]
    fn test_optimize_balances() {
        // Test trivial case
        let balances_1 = vec![
            UserBalance {
                username: "user1".to_string(),
                currency: "USD".to_string(),
                balance: 1000,
            },
            UserBalance {
                username: "user2".to_string(),
                currency: "USD".to_string(),
                balance: -1000,
            },
        ];

        // Expected debts
        let solution_1 = vec![Debt {
            debtor: "user2".to_string(),
            creditor: "user1".to_string(),
            currency: "USD".to_string(),
            amount: 1000,
        }];

        assert_eq!(optimize_debts(balances_1.clone()), solution_1);

        // Test more complex case of equal corresponding balances
        let balances_2 = vec![
            UserBalance {
                username: "user1".to_string(),
                currency: "USD".to_string(),
                balance: 1000,
            },
            UserBalance {
                username: "user2".to_string(),
                currency: "USD".to_string(),
                balance: -1000,
            },
            UserBalance {
                username: "user3".to_string(),
                currency: "USD".to_string(),
                balance: 1000,
            },
            UserBalance {
                username: "user4".to_string(),
                currency: "USD".to_string(),
                balance: -1000,
            },
            UserBalance {
                username: "user5".to_string(),
                currency: "USD".to_string(),
                balance: 0,
            },
        ];

        assert!(is_solution_correct(
            balances_2.clone(),
            optimize_debts(balances_2)
        ));

        // Test more complex case of different balances of random amounts
        let balances_3 = vec![
            UserBalance {
                username: "user1".to_string(),
                currency: "USD".to_string(),
                balance: 1200,
            },
            UserBalance {
                username: "user2".to_string(),
                currency: "USD".to_string(),
                balance: -670,
            },
            UserBalance {
                username: "user3".to_string(),
                currency: "USD".to_string(),
                balance: 513,
            },
            UserBalance {
                username: "user4".to_string(),
                currency: "USD".to_string(),
                balance: -300,
            },
            UserBalance {
                username: "user5".to_string(),
                currency: "USD".to_string(),
                balance: -743,
            },
        ];

        let solution_3 = optimize_debts(balances_3.clone());

        let expected_solution = vec![
            Debt {
                debtor: "user5".to_string(),
                creditor: "user1".to_string(),
                currency: "USD".to_string(),
                amount: 743,
            },
            Debt {
                debtor: "user2".to_string(),
                creditor: "user1".to_string(),
                currency: "USD".to_string(),
                amount: 457,
            },
            Debt {
                debtor: "user2".to_string(),
                creditor: "user3".to_string(),
                currency: "USD".to_string(),
                amount: 213,
            },
            Debt {
                debtor: "user4".to_string(),
                creditor: "user3".to_string(),
                currency: "USD".to_string(),
                amount: 300,
            },
        ];

        assert_eq!(solution_3, expected_solution);
        assert!(is_solution_correct(
            balances_3.clone(),
            optimize_debts(balances_3)
        ));

        // Test more complex example, using my own balances in my groups
        let balances_4 = vec![
            UserBalance {
                username: "user1".to_string(),
                currency: "USD".to_string(),
                balance: -29117,
            },
            UserBalance {
                username: "user2".to_string(),
                currency: "USD".to_string(),
                balance: -7386,
            },
            UserBalance {
                username: "user3".to_string(),
                currency: "USD".to_string(),
                balance: -11928,
            },
            UserBalance {
                username: "user4".to_string(),
                currency: "USD".to_string(),
                balance: -7388,
            },
            UserBalance {
                username: "user5".to_string(),
                currency: "USD".to_string(),
                balance: -3327,
            },
            UserBalance {
                username: "user6".to_string(),
                currency: "USD".to_string(),
                balance: 51679,
            },
            UserBalance {
                username: "user7".to_string(),
                currency: "USD".to_string(),
                balance: -70,
            },
            UserBalance {
                username: "user8".to_string(),
                currency: "USD".to_string(),
                balance: -3228,
            },
            UserBalance {
                username: "user9".to_string(),
                currency: "USD".to_string(),
                balance: -71,
            },
            UserBalance {
                username: "user10".to_string(),
                currency: "USD".to_string(),
                balance: 7467,
            },
            UserBalance {
                username: "user11".to_string(),
                currency: "USD".to_string(),
                balance: 3011,
            },
            UserBalance {
                username: "user12".to_string(),
                currency: "USD".to_string(),
                balance: 358,
            },
            UserBalance {
                username: "user13".to_string(),
                currency: "USD".to_string(),
                balance: 4787,
            },
            UserBalance {
                username: "user14".to_string(),
                currency: "USD".to_string(),
                balance: 16826,
            },
            UserBalance {
                username: "user15".to_string(),
                currency: "USD".to_string(),
                balance: 13847,
            },
            UserBalance {
                username: "user16".to_string(),
                currency: "USD".to_string(),
                balance: -16561,
            },
            UserBalance {
                username: "user17".to_string(),
                currency: "USD".to_string(),
                balance: -18899,
            },
            UserBalance {
                username: "user18".to_string(),
                currency: "USD".to_string(),
                balance: 11864,
            },
            UserBalance {
                username: "user19".to_string(),
                currency: "USD".to_string(),
                balance: -11864,
            },
            UserBalance {
                username: "user20".to_string(),
                currency: "USD".to_string(),
                balance: 0,
            },
        ];

        assert!(is_solution_correct(
            balances_4.clone(),
            optimize_debts(balances_4)
        ));
    }
}
