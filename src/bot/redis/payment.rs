use redis::{Commands, Connection, RedisResult};
use uuid::Uuid;

/* Payment CRUD Operations
 * Payment represents a payment entry, used in groups.
 * Payment comprises of a description, immutable datetime, creditor, numeric total,
 * and a list of debts (stored under a different key).
 * Has add, exists, get, update, and delete operations.
 */

const PAYMENT_KEY: &str = "payment";
const PAYMENT_DEBT_KEY: &str = "payment_debt";

// Debt is an abstraction containing a debtor (String) and the owed amount (i64)
pub type Debt = (String, i64);

// Payment contains all fields stored in Redis related to a single payment entry
#[derive(Debug, PartialEq)]
pub struct Payment {
    pub description: String,
    pub datetime: String,
    pub creditor: String,
    pub currency: String,
    pub total: i64,
    pub debts: Vec<Debt>,
}

// Adds a new payment to Redis
pub fn add_payment(con: &mut Connection, payment: &Payment) -> RedisResult<String> {
    let id = Uuid::new_v4().to_string();
    let main_key = format!("{PAYMENT_KEY}:{id}");
    con.hset(&main_key, "description", &payment.description)?;
    con.hset(&main_key, "datetime", &payment.datetime)?;
    con.hset(&main_key, "creditor", &payment.creditor)?;
    con.hset(&main_key, "currency", &payment.currency)?;
    con.hset(&main_key, "total", &payment.total)?;

    let debt_key = format!("{PAYMENT_DEBT_KEY}:{id}");
    for debt in &payment.debts {
        con.rpush(&debt_key, debt)?;
    }

    Ok(id)
}

// Gets a payment from Redis
pub fn get_payment(con: &mut Connection, payment_id: &str) -> RedisResult<Payment> {
    let main_key = format!("{PAYMENT_KEY}:{payment_id}");
    let description: String = con.hget(&main_key, "description")?;
    let datetime: String = con.hget(&main_key, "datetime")?;
    let creditor: String = con.hget(&main_key, "creditor")?;
    let currency: String = con.hget(&main_key, "currency")?;
    let total: i64 = con.hget(&main_key, "total")?;

    let debt_key = format!("{PAYMENT_DEBT_KEY}:{payment_id}");
    let debts: Vec<Debt> = con.lrange(&debt_key, 0, -1)?;

    let payment = Payment {
        description,
        datetime,
        creditor,
        currency,
        total,
        debts,
    };

    Ok(payment)
}

// Updates a payment in Redis
pub fn update_payment(
    con: &mut Connection,
    payment_id: &str,
    description: Option<&str>,
    creditor: Option<&str>,
    currency: Option<&str>,
    total: Option<&i64>,
    debts: Option<Vec<Debt>>,
) -> RedisResult<()> {
    let main_key = format!("{PAYMENT_KEY}:{payment_id}");
    if let Some(desc) = description {
        con.hset(&main_key, "description", desc)?;
    }
    if let Some(cred) = creditor {
        con.hset(&main_key, "creditor", cred)?;
    }
    if let Some(curr) = currency {
        con.hset(&main_key, "currency", curr)?;
    }
    if let Some(tot) = total {
        con.hset(&main_key, "total", tot)?;
    }
    if let Some(debt) = debts {
        let debt_key = format!("{PAYMENT_DEBT_KEY}:{payment_id}");
        con.del(&debt_key)?;
        for d in debt {
            con.rpush(&debt_key, d)?;
        }
    }

    Ok(())
}

// Deletes a payment from Redis
pub fn delete_payment(con: &mut Connection, payment_id: &str) -> RedisResult<()> {
    let main_key = format!("{PAYMENT_KEY}:{payment_id}");
    let debt_key = format!("{PAYMENT_DEBT_KEY}:{payment_id}");
    con.del(&main_key)?;
    con.del(&debt_key)?;

    Ok(())
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::bot::redis::connect::connect;

    #[test]
    fn test_add_get_payment() {
        let mut con = connect().unwrap();

        let description = "test_payment";
        let datetime = "2020-01-01T00:00:00Z";
        let creditor = "test_creditor";
        let currency = "USD";
        let total = 10000;
        let debts = vec![("test_debtor".to_string(), 10000)];
        let first_payment = Payment {
            description: description.to_string(),
            datetime: datetime.to_string(),
            creditor: creditor.to_string(),
            currency: currency.to_string(),
            total,
            debts: debts.clone(),
        };
        let payment_op = add_payment(&mut con, &first_payment);

        assert!(payment_op.is_ok());

        let payment_id = payment_op.unwrap();
        let payment = get_payment(&mut con, &payment_id);
        assert_eq!(payment.unwrap(), first_payment);

        delete_payment(&mut con, &payment_id).unwrap();
    }

    #[test]
    fn test_update_payment() {
        let mut con = connect().unwrap();

        let description = "test_payment";
        let datetime = "2020-01-01T00:00:00Z";
        let creditor = "test_creditor";
        let currency = "JPY";
        let total = 10000;
        let debts = vec![("test_debtor".to_string(), 10000)];
        let first_payment = Payment {
            description: description.to_string(),
            datetime: datetime.to_string(),
            creditor: creditor.to_string(),
            currency: currency.to_string(),
            total,
            debts: debts.clone(),
        };
        let payment_id = add_payment(&mut con, &first_payment).unwrap();

        let new_description = "new_test_payment";
        let new_creditor = "new_test_creditor";
        let new_currency = "USD";
        let new_total = 20000;
        let new_debts = vec![("new_test_debtor".to_string(), 20000)];

        let update_op = update_payment(
            &mut con,
            &payment_id,
            Some(new_description),
            Some(new_creditor),
            Some(new_currency),
            Some(&new_total),
            Some(new_debts.clone()),
        );

        assert!(update_op.is_ok());

        let payment = get_payment(&mut con, &payment_id);
        assert_eq!(
            payment.unwrap(),
            Payment {
                description: new_description.to_string(),
                datetime: datetime.to_string(),
                creditor: new_creditor.to_string(),
                currency: new_currency.to_string(),
                total: new_total,
                debts: new_debts.clone(),
            }
        );

        delete_payment(&mut con, &payment_id).unwrap();
    }

    #[test]
    fn test_delete_payment() {
        let mut con = connect().unwrap();

        let description = "test_payment";
        let datetime = "2020-01-01T00:00:00Z";
        let creditor = "test_creditor";
        let currency = "USD";
        let total = 10000;
        let debts = vec![("test_debtor".to_string(), 10000)];
        let payment_id = add_payment(
            &mut con,
            &Payment {
                description: description.to_string(),
                datetime: datetime.to_string(),
                creditor: creditor.to_string(),
                currency: currency.to_string(),
                total,
                debts: debts.clone(),
            },
        )
        .unwrap();
        assert!(delete_payment(&mut con, &payment_id).is_ok());
    }
}
