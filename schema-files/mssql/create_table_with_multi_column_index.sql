CREATE TABLE Product (
    [Id] UNIQUEIDENTIFIER NOT NULL PRIMARY KEY,
    [Name] NVARCHAR(255),
    [Color] NVARCHAR(255),
    [Size] NVARCHAR(255),
);

CREATE INDEX Vote_Name_Color_Size ON Product (Name, Color, Size);