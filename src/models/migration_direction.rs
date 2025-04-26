use super::ApplyOperation;

pub enum MigrationDirection {
    Forward,
    Backward,
}

impl From<ApplyOperation> for MigrationDirection {
    fn from(operation: ApplyOperation) -> Self {
        MigrationDirection::from(&operation)
    }
}

impl From<&ApplyOperation> for MigrationDirection {
    fn from(operation: &ApplyOperation) -> Self {
        match operation {
            ApplyOperation::Up | ApplyOperation::UpSingle | ApplyOperation::UpTo(_) => {
                MigrationDirection::Forward
            }
            ApplyOperation::Reset | ApplyOperation::DownSingle | ApplyOperation::DownTo(_) => {
                MigrationDirection::Backward
            }
        }
    }
}
