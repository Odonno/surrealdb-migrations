DEFINE TABLE OVERWRITE customer SCHEMALESS
    PERMISSIONS
        FOR select FULL
        FOR update WHERE id = $auth.id
        FOR create, delete NONE;

DEFINE FIELD OVERWRITE name ON customer TYPE string;
DEFINE FIELD OVERWRITE email ON customer TYPE string ASSERT string::is::email($value);
DEFINE FIELD OVERWRITE password ON customer TYPE string;
DEFINE FIELD OVERWRITE addresses ON customer TYPE array<record<address>>;

DEFINE INDEX OVERWRITE unique_email ON customer COLUMNS email UNIQUE;

DEFINE SCOPE OVERWRITE customer_scope
    SESSION 30d
    SIGNUP (
        CREATE customer
        SET
            name = $name,
            email = $email,
            password = crypto::argon2::generate($password)
    )
    SIGNIN (
        SELECT *
        FROM customer
        WHERE email = $email AND crypto::argon2::compare(password, $password)
    );
