CREATE TABLE Post (
    [Id] UNIQUEIDENTIFIER NOT NULL PRIMARY KEY,
    [Title] NVARCHAR(255),
    [Content] TEXT,
    [Status] NVARCHAR(50),
    [CreatedAt] DATETIME,
)
