use postgres::{Client, Row};

use crate::error::Error;
use super::dataformat::{DataFrame, Query};


const CONDITION_TIME_SPAN_AND_DEVICE: &str = "FROM Dataframe WHERE device_id = $1 AND $2 <=time_stamp AND time_stamp<=$3";
const CONDITION_TIME_OLDER_AND_DEVICE: &str = "FROM Dataframe WHERE device_id = $1 AND $2 < time_stamp OR $2 = time_stamp";
const CONDITION_TIME_YOUNGER_AND_DEVICE: &str = "FROM Dataframe WHERE device_id = $1 AND $2 > time_stamp OR $2 = time_stamp";
const CONDITION_TIME_NOW_AND_DEVICE: &str = "FROM Dataframe WHERE device_id = $1 AND $2 = time_stamp";

const SELECT_STATEMENT: &str = "SELECT * ";
const DELETE_STATEMENT: &str = "DELETE ";

pub fn get(query: &Query, client: &mut Client) -> Result<Vec<DataFrame>, Error> {
    let result: Result<Vec<Row>, Error> = match query {
        Query { device_id: Some(id), time_start: Some(start), time_end: Some(end), time_now: None } => client.query(&(SELECT_STATEMENT.to_string() + CONDITION_TIME_SPAN_AND_DEVICE), &[&id, &start, &end]).map_err(|err| Error::DatabaseQueryFailed(err.to_string())),
        Query { device_id: Some(id), time_start: Some(start), time_end: None, time_now: None } => client.query(&(SELECT_STATEMENT.to_string() + CONDITION_TIME_OLDER_AND_DEVICE), &[&id, &start]).map_err(|err| Error::DatabaseQueryFailed(err.to_string())),
        Query { device_id: Some(id), time_start: None, time_end: Some(end), time_now: None } => client.query(&(SELECT_STATEMENT.to_string() + CONDITION_TIME_YOUNGER_AND_DEVICE), &[&id, &end]).map_err(|err| Error::DatabaseQueryFailed(err.to_string())),
        Query { device_id: Some(id), time_start: None, time_end: None, time_now: Some(now) } => client.query(&(SELECT_STATEMENT.to_string() + CONDITION_TIME_NOW_AND_DEVICE), &[&id, &now]).map_err(|err| Error::DatabaseQueryFailed(err.to_string())),
        _ => Err(Error::DatabaseQueryNotSupported)
    };

    Ok(result?.iter().map(|row| {
            let temp: i16 = row.get("temp");
            let rpm: i32 = row.get("rpm");
            let device_id: i32 = row.get("device_id");
            let time_stamp: i32 = row.get("time_stamp");

            DataFrame { temp, rpm, device_id, time_stamp }
        }).collect()
    )
}

pub fn delete(query: &Query, client: &mut Client) -> Result<(), Error> {
    if let Err(err)  = match query {
        Query { device_id: Some(id), time_start: Some(start), time_end: Some(end), time_now: None } => client.execute(&(DELETE_STATEMENT.to_string() + CONDITION_TIME_SPAN_AND_DEVICE), &[&id, &start, &end]).map_err(|err| Error::DatabaseDeletionError(err.to_string())),
        Query { device_id: Some(id), time_start: Some(start), time_end: None, time_now: None } => client.execute(&(DELETE_STATEMENT.to_string() + CONDITION_TIME_OLDER_AND_DEVICE), &[&id, &start]).map_err(|err| Error::DatabaseDeletionError(err.to_string())),
        Query { device_id: Some(id), time_start: None, time_end: Some(end), time_now: None } => client.execute(&(DELETE_STATEMENT.to_string() + CONDITION_TIME_YOUNGER_AND_DEVICE), &[&id, &end]).map_err(|err| Error::DatabaseDeletionError(err.to_string())),
        Query { device_id: Some(id), time_start: None, time_end: None, time_now: Some(now) } => client.execute(&(DELETE_STATEMENT.to_string() + CONDITION_TIME_NOW_AND_DEVICE), &[&id, &now]).map_err(|err| Error::DatabaseDeletionError(err.to_string())),
        _ => Err(Error::DatabaseDeletionError("Unsupported deletion operation".to_string()))
    } {
        Err(err)
    } else {
        Ok(())
    }
}

pub fn update(query: &Query, dataframe: &DataFrame, client: &mut Client) -> Result<(), Error> {
    // Destructure the dataframe to extract the values to be updated
    let DataFrame { temp, rpm, time_stamp, .. } = dataframe;

    // Match on the query to determine the condition
    let result = match query {
        Query { device_id: Some(id), time_start: Some(start), time_end: Some(end), time_now: None } => {
            let update_query: &str = "UPDATE Dataframe SET temp = $4, rpm = $5, time_stamp = $6 WHERE device_id = $1 AND $2 <= time_stamp AND time_stamp <= $3";
            client.execute(update_query, &[&id, &start, &end, &temp, &rpm, &time_stamp])
        },
        Query { device_id: Some(id), time_start: Some(start), time_end: None, time_now: None } => {
            let update_query: &str = "UPDATE Dataframe SET temp = $3, rpm = $4, time_stamp = $5 WHERE device_id = $1 AND $2 < time_stamp OR $2 time_stamp = $2";
            client.execute(update_query, &[&id, &start, &temp, &rpm, &time_stamp])
        },
        Query { device_id: Some(id), time_start: None, time_end: Some(end), time_now: None } => {
            let update_query: &str = "UPDATE Dataframe SET temp = $3, rpm = $4, time_stamp = $5 WHERE device_id = $1 AND time_stamp < $2 OR time_stamp = $2";
            client.execute(update_query, &[&id, &end, &temp, &rpm, &time_stamp])
        },
        Query { device_id: Some(id), time_start: None, time_end: None, time_now: Some(now) } => {
            let update_query: &str = "UPDATE Dataframe SET temp = $3, rpm = $4, time_stamp = $5 WHERE device_id = $1 AND time_stamp = $2";
            client.execute(update_query, &[&id, &now, &temp, &rpm, &time_stamp])
        },
        _ => return Err(Error::DatabaseUpdateError("Unsupported update operation".to_string())),
    };

    // Check the result and return appropriately
    match result {
        Ok(_) => Ok(()),
        Err(err) => Err(Error::DatabaseUpdateError(err.to_string())),
    }
}
