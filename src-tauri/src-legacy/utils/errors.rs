use std::fmt;

#[derive(Debug)]
pub struct LockError(pub String);

impl fmt::Display for LockError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "锁获取失败: {}", self.0)
    }
}

#[derive(Debug)]
pub struct DatabaseError(pub String);

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "数据库错误: {}", self.0)
    }
}
