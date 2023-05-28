DEFINE TABLE branch SCHEMAFULL;

DEFINE FIELD name ON branch TYPE string;
DEFINE FIELD from_ns ON branch TYPE string;
DEFINE FIELD from_db ON branch TYPE string; 
DEFINE FIELD created_at ON branch TYPE datetime VALUE $before OR time::now();