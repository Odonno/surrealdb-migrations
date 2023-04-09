DEFINE TABLE register_customer SCHEMALESS;

DEFINE FIELD name ON register_customer TYPE string;
DEFINE FIELD email ON register_customer TYPE string;

DEFINE EVENT register_customer ON TABLE register_customer WHEN $before == NONE THEN {
    CREATE customer SET name = $after.name, email = $after.email;
};