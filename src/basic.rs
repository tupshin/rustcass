 /*
Copyright (c) 2014 DataStax

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

#![feature(globs)]
extern crate libc;
use std::c_str::CString;
use cassandra::*;
mod cassandra;

struct Basic {
  bln: bool,
  flt: f32,
  dbl: f64,
  i32: i32,
  i64: i64
}


unsafe fn print_error(future:*mut CassFuture) {
  let message = cass_future_error_message(future);
  println!("Error: {}", message.data);//FIXME stderr
}


unsafe fn create_cluster() -> *mut CassCluster {
  let contact_points = ["127.0.0.1"];
  let cluster:*mut CassCluster = cass_cluster_new();
  for contact_point in contact_points.iter() {
    let  c:&'static str=*contact_point;
    let cstr=c.to_c_str();
    cass_cluster_setopt( cluster, CASS_OPTION_CONTACT_POINTS,  cstr.as_ptr() as *const libc::c_void, cstr.len() as  u64);
  }
  return cluster;
}

unsafe fn connect_session(cluster:*mut CassCluster)-> (CassError,*mut CassSession) {
  let future:*mut CassFuture = cass_cluster_connect(cluster);
  cass_future_wait(future);
  let rc:CassError = cass_future_error_code(future);
  let session:*mut CassSession = cass_future_get_session(future);

  cass_future_free(future);

  return (rc,session);
}

unsafe fn execute_query(session:*mut CassSession, query:CString) -> CassError {
  let statement:*mut CassStatement = cass_statement_new(cass_string_init(query.as_ptr() as *const i8), 0, CASS_CONSISTENCY_ONE);

  let future:*mut CassFuture = cass_session_execute(session, statement);
  cass_future_wait(future);

  let rc:CassError = cass_future_error_code(future);
  if rc != CASS_OK {
    print_error(future);
  }

  cass_future_free(future);
  cass_statement_free(statement);

  return rc;
}

unsafe fn insert_into_basic(session:*mut CassSession, key:CString, basic:Basic) -> CassError {
  let rawString = "INSERT INTO examples.basic (key, bln, flt, dbl, i32, i64) VALUES (?, ?, ?, ?, ?, ?);";
  let query:CassString = cass_string_init(rawString.as_ptr() as *const i8);

  let statement:*mut CassStatement = cass_statement_new(query, 6, CASS_CONSISTENCY_ONE);
  cass_statement_bind_string(statement, 0, cass_string_init(key.as_ptr() as *const i8));
  cass_statement_bind_bool(statement, 1, basic.bln as u32);
  cass_statement_bind_float(statement, 2, basic.flt);
  cass_statement_bind_double(statement, 3, basic.dbl);
  cass_statement_bind_int32(statement, 4, basic.i32);
  cass_statement_bind_int64(statement, 5, basic.i64);

  let future:*mut CassFuture = cass_session_execute(session, statement);
  cass_future_wait(future);

  let rc:CassError = cass_future_error_code(future);
  if rc != CASS_OK {
    print_error(future);
  }

  cass_future_free(future);
  cass_statement_free(statement);

  return rc;
}

unsafe fn select_from_basic(session:*mut CassSession, key:CString, basic:Basic) -> CassError {
let rawString = "SELECT * FROM examples.basic WHERE key = ?;";
let query:CassString = cass_string_init(rawString.to_c_str().as_ptr() as *const i8);

  let statement:*mut CassStatement = cass_statement_new(query, 1, CASS_CONSISTENCY_ONE);

  cass_statement_bind_string(statement, 0, cass_string_init(key.as_ptr() as *const i8));

  let future:*mut CassFuture = cass_session_execute(session, statement);
  cass_future_wait(future);

  let rc:CassError = cass_future_error_code(future);
  if rc != CASS_OK {
    print_error(future);
  } else {
    let result:*const CassResult = cass_future_get_result(future);
    let iterator:*mut CassIterator = cass_iterator_from_result(result);

    if cass_iterator_next(iterator) > 0 {
      let row:*const CassRow = cass_iterator_get_row(iterator);
      cass_value_get_bool(cass_row_get_column(row, 1), basic.bln as *mut u32);
//FIXME. the lines below generate compiler errors
//      cass_value_get_double(cass_row_get_column(row, 2), basic.dbl as *mut f64);
//      cass_value_get_float(cass_row_get_column(row, 3), basic.flt as *mut f32);
      cass_value_get_int32(cass_row_get_column(row, 4), basic.i32 as *mut i32);
      cass_value_get_int64(cass_row_get_column(row, 5), basic.i64 as *mut i64);
    }
    cass_result_free(result);
    cass_iterator_free(iterator);
  }
  cass_future_free(future);
  cass_statement_free(statement);

  return rc;
}

fn main()  {
  unsafe {
    let cluster:*mut CassCluster = create_cluster();

    let input:Basic = Basic{bln:cass_true > 0, dbl:0.001f64, flt:0.0002f32, i32:1, i64:2 };
    let output:Basic=  Basic{bln:false, dbl:0.0f64,flt:0.0f32, i32:1, i64:2};

    let (rc,session) = connect_session(cluster);
    if rc != CASS_OK {
      println!("rc={}",rc);
      return
    }
    let rawString = "CREATE KEYSPACE examples WITH replication = { 'class': 'SimpleStrategy', 'replication_factor': '1' };";
    execute_query(session, rawString.to_c_str());

    let create_table_string = "CREATE TABLE examples.basic (key text, bln boolean, flt float, dbl double, i32 int, i64 bigint, PRIMARY KEY (key));";

    execute_query(session, create_table_string.to_c_str());

    insert_into_basic(session, "test".to_c_str(), input);
    select_from_basic(session, "test".to_c_str(), output);

  // assert!(input.bln == output.bln);
  // assert!(input.flt == output.flt);
  // assert!(input.dbl == output.dbl);
  // assert!(input.i32 == output.i32);
  // assert!(input.i64 == output.i64);
  //
    let close_future = cass_session_close(session);
    cass_future_wait(close_future);
    cass_cluster_free(cluster);
  }
}
