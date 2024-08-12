CREATE TABLE Vote (
    [Id] UNIQUEIDENTIFIER NOT NULL PRIMARY KEY,
    [Username] NVARCHAR(255),
    [Movie] NVARCHAR(255)
);

CREATE UNIQUE INDEX Vote_Username_Movie_Unique ON Vote (Username, Movie);