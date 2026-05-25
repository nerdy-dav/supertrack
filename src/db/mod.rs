use rusqlite::{Connection, Result, params};
use std::sync::Mutex;

pub struct Database {
    conn: Mutex<Connection>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Business {
    pub id: i64,
    pub name: String,
    pub abn: String,
    pub state: String,
    pub pay_frequency: String,
    pub payroll_software: Option<String>,
    pub clearing_house: Option<String>,
    pub clearing_house_lag_days: i64,
    pub sgc_rate: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Employee {
    pub id: i64,
    pub name: String,
    pub employment_type: String,
    pub super_fund_name: String,
    pub super_fund_usi: Option<String>,
    pub member_number: Option<String>,
    pub ote_weekly: f64,
    pub active: bool,
    pub start_date: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PayRun {
    pub id: i64,
    pub pay_date: String,
    pub period_start: String,
    pub period_end: String,
    pub total_wages: f64,
    pub total_super_owing: f64,
    pub super_deadline: String,
    pub stp_reference: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SuperPayment {
    pub id: i64,
    pub pay_run_id: i64,
    pub payment_date: String,
    pub clearing_house_receipt: Option<String>,
    pub amount_paid: f64,
    pub fund_receipt_confirmed: bool,
    pub fund_receipt_date: Option<String>,
    pub status: String,
}

impl Database {
    pub fn new(path: &str) -> anyhow::Result<Self> {
        let conn = Connection::open(path)?;
        Ok(Database { conn: Mutex::new(conn) })
    }

    pub fn migrate(&self) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch("
            PRAGMA journal_mode=WAL;

            CREATE TABLE IF NOT EXISTS business (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                abn TEXT NOT NULL DEFAULT '',
                state TEXT NOT NULL DEFAULT 'VIC',
                pay_frequency TEXT NOT NULL DEFAULT 'fortnightly',
                payroll_software TEXT,
                clearing_house TEXT,
                clearing_house_lag_days INTEGER DEFAULT 1,
                sgc_rate REAL DEFAULT 0.12,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS employees (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                employment_type TEXT NOT NULL DEFAULT 'full-time',
                super_fund_name TEXT NOT NULL DEFAULT '',
                super_fund_usi TEXT,
                member_number TEXT,
                ote_weekly REAL NOT NULL DEFAULT 0,
                active INTEGER DEFAULT 1,
                start_date TEXT NOT NULL DEFAULT (date('now')),
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS pay_runs (
                id INTEGER PRIMARY KEY,
                pay_date TEXT NOT NULL,
                period_start TEXT NOT NULL,
                period_end TEXT NOT NULL,
                total_wages REAL NOT NULL DEFAULT 0,
                total_super_owing REAL NOT NULL DEFAULT 0,
                super_deadline TEXT NOT NULL,
                stp_reference TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS pay_run_items (
                id INTEGER PRIMARY KEY,
                pay_run_id INTEGER REFERENCES pay_runs(id),
                employee_id INTEGER REFERENCES employees(id),
                gross_wages REAL NOT NULL DEFAULT 0,
                super_calculated REAL NOT NULL DEFAULT 0,
                super_override REAL,
                override_reason TEXT
            );

            CREATE TABLE IF NOT EXISTS super_payments (
                id INTEGER PRIMARY KEY,
                pay_run_id INTEGER REFERENCES pay_runs(id),
                payment_date TEXT NOT NULL,
                clearing_house_receipt TEXT,
                amount_paid REAL NOT NULL DEFAULT 0,
                fund_receipt_confirmed INTEGER DEFAULT 0,
                fund_receipt_date TEXT,
                status TEXT NOT NULL DEFAULT 'pending',
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
        ")?;
        Ok(())
    }

    // Business
    pub fn get_business(&self) -> Option<Business> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT id, name, abn, state, pay_frequency, payroll_software, clearing_house, clearing_house_lag_days, sgc_rate FROM business LIMIT 1",
            [],
            |row| Ok(Business {
                id: row.get(0)?,
                name: row.get(1)?,
                abn: row.get(2)?,
                state: row.get(3)?,
                pay_frequency: row.get(4)?,
                payroll_software: row.get(5)?,
                clearing_house: row.get(6)?,
                clearing_house_lag_days: row.get(7)?,
                sgc_rate: row.get(8)?,
            })
        ).ok()
    }

    pub fn upsert_business(&self, b: &Business) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO business (id, name, abn, state, pay_frequency, payroll_software, clearing_house, clearing_house_lag_days, sgc_rate, created_at)
             VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, datetime('now'))",
            params![b.name, b.abn, b.state, b.pay_frequency, b.payroll_software, b.clearing_house, b.clearing_house_lag_days, b.sgc_rate],
        )?;
        Ok(())
    }

    // Employees
    pub fn list_employees(&self) -> Vec<Employee> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, employment_type, super_fund_name, super_fund_usi, member_number, ote_weekly, active, start_date FROM employees ORDER BY name"
        ).unwrap();
        stmt.query_map([], |row| Ok(Employee {
            id: row.get(0)?,
            name: row.get(1)?,
            employment_type: row.get(2)?,
            super_fund_name: row.get(3)?,
            super_fund_usi: row.get(4)?,
            member_number: row.get(5)?,
            ote_weekly: row.get(6)?,
            active: row.get::<_, i64>(7)? == 1,
            start_date: row.get(8)?,
        })).unwrap().filter_map(Result::ok).collect()
    }

    pub fn create_employee(&self, e: &Employee) -> anyhow::Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO employees (name, employment_type, super_fund_name, super_fund_usi, member_number, ote_weekly, active, start_date) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
            params![e.name, e.employment_type, e.super_fund_name, e.super_fund_usi, e.member_number, e.ote_weekly, e.active as i64, e.start_date],
        )?;
        Ok(conn.last_insert_rowid())
    }

    // Pay runs
    pub fn list_pay_runs(&self) -> Vec<PayRun> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, pay_date, period_start, period_end, total_wages, total_super_owing, super_deadline, stp_reference, created_at FROM pay_runs ORDER BY pay_date DESC"
        ).unwrap();
        stmt.query_map([], |row| Ok(PayRun {
            id: row.get(0)?,
            pay_date: row.get(1)?,
            period_start: row.get(2)?,
            period_end: row.get(3)?,
            total_wages: row.get(4)?,
            total_super_owing: row.get(5)?,
            super_deadline: row.get(6)?,
            stp_reference: row.get(7)?,
            created_at: row.get(8)?,
        })).unwrap().filter_map(Result::ok).collect()
    }

    pub fn get_pay_run(&self, id: i64) -> Option<PayRun> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT id, pay_date, period_start, period_end, total_wages, total_super_owing, super_deadline, stp_reference, created_at FROM pay_runs WHERE id=?1",
            params![id],
            |row| Ok(PayRun {
                id: row.get(0)?,
                pay_date: row.get(1)?,
                period_start: row.get(2)?,
                period_end: row.get(3)?,
                total_wages: row.get(4)?,
                total_super_owing: row.get(5)?,
                super_deadline: row.get(6)?,
                stp_reference: row.get(7)?,
                created_at: row.get(8)?,
            })
        ).ok()
    }

    pub fn create_pay_run(&self, pr: &PayRun) -> anyhow::Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO pay_runs (pay_date, period_start, period_end, total_wages, total_super_owing, super_deadline, stp_reference) VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![pr.pay_date, pr.period_start, pr.period_end, pr.total_wages, pr.total_super_owing, pr.super_deadline, pr.stp_reference],
        )?;
        Ok(conn.last_insert_rowid())
    }

    // Super payments
    pub fn get_payment_for_run(&self, pay_run_id: i64) -> Option<SuperPayment> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT id, pay_run_id, payment_date, clearing_house_receipt, amount_paid, fund_receipt_confirmed, fund_receipt_date, status FROM super_payments WHERE pay_run_id=?1 LIMIT 1",
            params![pay_run_id],
            |row| Ok(SuperPayment {
                id: row.get(0)?,
                pay_run_id: row.get(1)?,
                payment_date: row.get(2)?,
                clearing_house_receipt: row.get(3)?,
                amount_paid: row.get(4)?,
                fund_receipt_confirmed: row.get::<_, i64>(5)? == 1,
                fund_receipt_date: row.get(6)?,
                status: row.get(7)?,
            })
        ).ok()
    }

    pub fn create_payment(&self, p: &SuperPayment) -> anyhow::Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO super_payments (pay_run_id, payment_date, clearing_house_receipt, amount_paid, fund_receipt_confirmed, fund_receipt_date, status) VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![p.pay_run_id, p.payment_date, p.clearing_house_receipt, p.amount_paid, p.fund_receipt_confirmed as i64, p.fund_receipt_date, p.status],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn list_all_payments(&self) -> Vec<(PayRun, Option<SuperPayment>)> {
        let runs = self.list_pay_runs();
        runs.into_iter().map(|run| {
            let payment = self.get_payment_for_run(run.id);
            (run, payment)
        }).collect()
    }
}
