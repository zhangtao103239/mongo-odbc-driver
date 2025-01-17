use crate::api::{definitions::*, errors::ODBCError};
use odbc_sys::{HDbc, HEnv, HStmt, Handle, Len, Pointer, ULen, USmallInt};
use std::{borrow::BorrowMut, collections::HashSet, ptr::null_mut, sync::RwLock};

#[derive(Debug)]
pub enum MongoHandle {
    Env(RwLock<Env>),
    Connection(RwLock<Connection>),
    Statement(RwLock<Statement>),
}

impl MongoHandle {
    pub fn as_env(&self) -> Option<&RwLock<Env>> {
        match self {
            MongoHandle::Env(e) => Some(e),
            _ => None,
        }
    }

    pub fn as_connection(&self) -> Option<&RwLock<Connection>> {
        match self {
            MongoHandle::Connection(c) => Some(c),
            _ => None,
        }
    }

    pub fn as_statement(&self) -> Option<&RwLock<Statement>> {
        match self {
            MongoHandle::Statement(s) => Some(s),
            _ => None,
        }
    }

    /// add_diag_info appends a new ODBCError object to the `errors` field.
    pub fn add_diag_info(&mut self, error: ODBCError) {
        match self {
            MongoHandle::Env(e) => {
                let mut env_contents = (*e).write().unwrap();
                env_contents.errors.push(error);
            }
            MongoHandle::Connection(c) => {
                let mut dbc_contents = (*c).write().unwrap();
                dbc_contents.errors.push(error);
            }
            MongoHandle::Statement(s) => {
                let mut stmt_contents = (*s).write().unwrap();
                stmt_contents.errors.push(error);
            }
        }
    }

    pub fn clear_diagnostics(&mut self) {
        match self {
            MongoHandle::Env(e) => {
                let mut env_contents = (*e).write().unwrap();
                env_contents.errors.clear();
            }
            MongoHandle::Connection(c) => {
                let mut dbc_contents = (*c).write().unwrap();
                dbc_contents.errors.clear();
            }
            MongoHandle::Statement(s) => {
                let mut stmt_contents = (*s).write().unwrap();
                stmt_contents.errors.clear();
            }
        }
    }
}

pub type MongoHandleRef = &'static mut MongoHandle;

impl From<Handle> for MongoHandleRef {
    fn from(handle: Handle) -> Self {
        unsafe { (*(handle as *mut MongoHandle)).borrow_mut() }
    }
}

impl From<HEnv> for MongoHandleRef {
    fn from(handle: HEnv) -> Self {
        unsafe { (*(handle as *mut MongoHandle)).borrow_mut() }
    }
}

impl From<HStmt> for MongoHandleRef {
    fn from(handle: HStmt) -> Self {
        unsafe { (*(handle as *mut MongoHandle)).borrow_mut() }
    }
}

impl From<HDbc> for MongoHandleRef {
    fn from(handle: HDbc) -> Self {
        unsafe { (*(handle as *mut MongoHandle)).borrow_mut() }
    }
}

#[derive(Debug)]
pub struct Env {
    // attributes for this Env. We box the attributes so that the MongoHandle type
    // remains fairly small regardless of underlying handle type.
    pub attributes: Box<EnvAttributes>,
    // state of this Env
    pub state: EnvState,
    pub connections: HashSet<*mut MongoHandle>,
    pub errors: Vec<ODBCError>,
}

impl Env {
    pub fn with_state(state: EnvState) -> Self {
        Self {
            attributes: Box::new(EnvAttributes::default()),
            state,
            connections: HashSet::new(),
            errors: vec![],
        }
    }
}

#[derive(Debug)]
pub struct EnvAttributes {
    pub odbc_ver: OdbcVersion,
    pub output_nts: SqlBool,
    pub connection_pooling: ConnectionPooling,
    pub cp_match: CpMatch,
}

impl Default for EnvAttributes {
    fn default() -> Self {
        Self {
            odbc_ver: OdbcVersion::Odbc3_80,
            output_nts: SqlBool::True,
            connection_pooling: ConnectionPooling::Off,
            cp_match: CpMatch::Strict,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum EnvState {
    Allocated,
    ConnectionAllocated,
}

#[derive(Debug)]
pub struct Connection {
    // type of this handle for runtime checking purposes.
    // Pointer to the Env from which
    // this Connection was allocated
    pub env: *mut MongoHandle,
    // all the possible Connection settings
    pub _attributes: Box<ConnectionAttributes>,
    // state of this connection
    pub state: ConnectionState,
    // MongoDB Client for issuing commands
    // pub client: Option<MongoClient>,
    // all Statements allocated from this Connection
    pub statements: HashSet<*mut MongoHandle>,
    pub errors: Vec<ODBCError>,
}

#[derive(Debug, Default)]
pub struct ConnectionAttributes {
    pub current_db: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ConnectionState {
    Allocated,
    _ConnectionFunctionNeedsDataEnv,
    Connected,
    StatementAllocated,
    _TransactionInProgress,
}

impl Connection {
    pub fn with_state(env: *mut MongoHandle, state: ConnectionState) -> Self {
        Self {
            env,
            _attributes: Box::new(ConnectionAttributes::default()),
            state,
            statements: HashSet::new(),
            errors: vec![],
        }
    }
}

#[derive(Debug)]
pub struct Statement {
    pub connection: *mut MongoHandle,
    pub attributes: Box<StatementAttributes>,
    pub state: StatementState,
    //pub cursor: Option<Box<Peekable<Cursor>>>,
    pub errors: Vec<ODBCError>,
}

#[derive(Debug)]
pub struct StatementAttributes {
    pub app_row_desc: Pointer,
    pub app_param_desc: Pointer,
    pub async_enable: AsyncEnable,
    pub async_stmt_event: Pointer,
    pub cursor_scrollable: CursorScrollable,
    pub cursor_sensitivity: CursorSensitivity,
    pub concurrency: Concurrency,
    pub cursor_type: CursorType,
    pub enable_auto_ipd: SqlBool,
    pub fetch_bookmark_ptr: *mut Len,
    pub imp_row_desc: Pointer,
    pub imp_param_desc: Pointer,
    pub max_length: ULen,
    pub max_rows: ULen,
    pub no_scan: NoScan,
    pub param_bind_offset_ptr: *mut ULen,
    pub param_bind_type: ULen,
    pub param_operation_ptr: *mut USmallInt,
    pub param_processed_ptr: *mut ULen,
    pub param_status_ptr: *mut USmallInt,
    pub paramset_size: ULen,
    pub query_timeout: ULen,
    pub retrieve_data: RetrieveData,
    pub row_array_size: ULen,
    pub row_bind_offset_ptr: *mut ULen,
    pub row_bind_type: ULen,
    pub row_number: ULen,
    pub row_operation_ptr: *mut USmallInt,
    pub row_status_ptr: *mut USmallInt,
    pub rows_fetched_ptr: *mut ULen,
    pub simulate_cursor: ULen,
    pub use_bookmarks: UseBookmarks,
}

#[derive(Debug, PartialEq, Eq)]
pub enum StatementState {
    Allocated,
    _Prepared,
    _PreparedHasResultSet,
    _ExecutedNoResultSet,
    _ExecutedHasResultSet,
    _CursorFetchSet,
    _CursorExtendedFetchSet,
    _FunctionNeedsDataNoParam,
    _FunctionNeedsDataNoPut,
    _FunctionNeedsDataPutCalled,
    _Executing,
    _AsyncCancelled,
}

impl Statement {
    pub fn with_state(connection: *mut MongoHandle, state: StatementState) -> Self {
        Self {
            connection,
            state,
            attributes: Box::new(StatementAttributes {
                app_row_desc: null_mut(),
                app_param_desc: null_mut(),
                async_enable: AsyncEnable::Off,
                async_stmt_event: null_mut(),
                cursor_scrollable: CursorScrollable::NonScrollable,
                cursor_sensitivity: CursorSensitivity::Insensitive,
                concurrency: Concurrency::ReadOnly,
                cursor_type: CursorType::ForwardOnly,
                enable_auto_ipd: SqlBool::False,
                fetch_bookmark_ptr: null_mut(),
                imp_row_desc: null_mut(),
                imp_param_desc: null_mut(),
                max_length: 0,
                max_rows: 0,
                no_scan: NoScan::Off,
                param_bind_offset_ptr: null_mut(),
                param_bind_type: BindType::BindByColumn as usize,
                param_operation_ptr: null_mut(),
                param_processed_ptr: null_mut(),
                param_status_ptr: null_mut(),
                paramset_size: 0,
                query_timeout: 0,
                retrieve_data: RetrieveData::Off,
                row_array_size: 1,
                row_bind_offset_ptr: null_mut(),
                row_bind_type: BindType::BindByColumn as usize,
                row_number: 0,
                row_operation_ptr: null_mut(),
                row_status_ptr: null_mut(),
                rows_fetched_ptr: null_mut(),
                simulate_cursor: SimulateCursor::NonUnique as usize,
                use_bookmarks: UseBookmarks::Off,
            }),
            errors: vec![],
        }
    }
}
