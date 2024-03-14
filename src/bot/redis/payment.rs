use super::connect::connect;
use redis::Commands;
use uuid::Uuid;

const PAYMENT_KEY: &str = "payment";
const PAYMENT_DEBT_KEY: &str = "payment_debt";

// Debt is an abstraction containing a debtor (String) and the owed amount (i32)
type Debt = (String, i32);

// Payment contains all fields stored in Redis related to a single payment entry
type Payment = (String, String, String, i32, Vec<Debt>);

// Adds a new payment to Redis
pub fn add_payment(
    description: &str,
    datetime: &str,
    creditor: &str,
    total: i32,
    debts: Vec<Debt>,
) -> redis::RedisResult<String> {
    let mut con = connect();

    // Add main fields
    let id = Uuid::new_v4().to_string();
    let main_key = format!("{PAYMENT_KEY}:{id}");
    con.hset(&main_key, "description", description)?;
    con.hset(&main_key, "datetime", datetime)?;
    con.hset(&main_key, "creditor", creditor)?;
    con.hset(&main_key, "total", total)?;

    let debt_key = format!("{PAYMENT_DEBT_KEY}:{id}");
    for debt in debts {
        con.rpush(&debt_key, debt)?;
    }

    Ok(id)
}

// Gets a payment from Redis
pub fn get_payment(payment_id: &str) -> redis::RedisResult<Payment> {
    let mut con = connect();

    let main_key = format!("{PAYMENT_KEY}:{payment_id}");
    let description: String = con.hget(&main_key, "description")?;
    let datetime: String = con.hget(&main_key, "datetime")?;
    let creditor: String = con.hget(&main_key, "creditor")?;
    let total: i32 = con.hget(&main_key, "total")?;

    let debt_key = format!("{PAYMENT_DEBT_KEY}:{payment_id}");
    let debts: Vec<Debt> = con.lrange(&debt_key, 0, -1)?;

    Ok((description, datetime, creditor, total, debts))
}

// Updates a payment in Redis
pub fn update_payment(
    payment_id: &str,
    description: Option<&str>,
    creditor: Option<&str>,
    total: Option<&i32>,
    debts: Option<Vec<Debt>>,
) -> redis::RedisResult<()> {
    let mut con = connect();

    let main_key = format!("{PAYMENT_KEY}:{payment_id}");
    if let Some(desc) = description {
        con.hset(&main_key, "description", desc)?;
    }
    if let Some(cred) = creditor {
        con.hset(&main_key, "creditor", cred)?;
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
// Mainly for testing purposes
// In application, no real need to delete keys
pub fn delete_payment(payment_id: &str) -> redis::RedisResult<()> {
    let mut con = connect();
    let main_key = format!("{PAYMENT_KEY}:{payment_id}");
    let debt_key = format!("{PAYMENT_DEBT_KEY}:{payment_id}");
    con.del(&main_key)?;
    con.del(&debt_key)?;

    Ok(())
}

// Tests
#[cfg(test)]
mod tests {
    use super::add_payment;
    use super::delete_payment;
    use super::get_payment;
    use super::update_payment;

    #[test]
    fn test_add_get_payment() {
        let description = "test_payment";
        let datetime = "2020-01-01T00:00:00Z";
        let creditor = "test_creditor";
        let total = 100;
        let debts = vec![("test_debtor".to_string(), 50)];
        let payment_op = add_payment(description, datetime, creditor, total, debts.clone());

        assert!(payment_op.is_ok());

        let payment_id = payment_op.unwrap();
        let payment = get_payment(&payment_id);
        assert_eq!(
            payment.unwrap(),
            (
                description.to_string(),
                datetime.to_string(),
                creditor.to_string(),
                total,
                debts
            )
        );

        delete_payment(&payment_id).unwrap();
    }

    #[test]
    fn test_update_payment() {
        let description = "test_payment";
        let datetime = "2020-01-01T00:00:00Z";
        let creditor = "test_creditor";
        let total = 100;
        let debts = vec![("test_debtor".to_string(), 50)];
        let payment_id =
            add_payment(description, datetime, creditor, total, debts.clone()).unwrap();

        let new_description = "new_test_payment";
        let new_creditor = "new_test_creditor";
        let new_total = 200;
        let new_debts = vec![("new_test_debtor".to_string(), 100)];

        let update_op = update_payment(
            &payment_id,
            Some(new_description),
            Some(new_creditor),
            Some(&new_total),
            Some(new_debts.clone()),
        );

        assert!(update_op.is_ok());

        let payment = get_payment(&payment_id);
        assert_eq!(
            payment.unwrap(),
            (
                new_description.to_string(),
                datetime.to_string(),
                new_creditor.to_string(),
                new_total,
                new_debts
            )
        );
    }

    #[test]
    fn test_delete_payment() {
        let description = "test_payment";
        let datetime = "2020-01-01T00:00:00Z";
        let creditor = "test_creditor";
        let total = 100;
        let debts = vec![("test_debtor".to_string(), 50)];
        let payment_id =
            add_payment(description, datetime, creditor, total, debts.clone()).unwrap();
        assert!(delete_payment(&payment_id).is_ok());
    }
}
