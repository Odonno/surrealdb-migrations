DEFINE TABLE customer SCHEMALESS
    PERMISSIONS
        FOR select FULL
        FOR update WHERE id = $auth.id
        FOR create, delete NONE;

DEFINE FIELD name ON customer TYPE string;
DEFINE FIELD email ON customer TYPE string;
DEFINE FIELD password ON customer TYPE string ASSERT $value != NONE;
DEFINE FIELD addresses ON customer TYPE array;
DEFINE FIELD addresses.* ON customer TYPE record(address);

DEFINE INDEX unique_email ON customer COLUMNS email UNIQUE;

DEFINE SCOPE customer_scope
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