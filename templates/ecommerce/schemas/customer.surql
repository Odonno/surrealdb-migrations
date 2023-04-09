DEFINE TABLE customer SCHEMALESS;

DEFINE FIELD name ON customer TYPE string;
DEFINE FIELD email ON customer TYPE string;
DEFINE FIELD address ON customer TYPE record(address);