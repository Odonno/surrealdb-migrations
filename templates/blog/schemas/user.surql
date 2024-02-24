DEFINE TABLE user SCHEMAFULL
    PERMISSIONS
        FOR select FULL
        FOR update WHERE id = $auth.id
        FOR create, delete NONE;

DEFINE FIELD username ON user TYPE string;
DEFINE FIELD email ON user TYPE string ASSERT string::is::email($value);
DEFINE FIELD password ON user TYPE string;
DEFINE FIELD registered_at ON user TYPE datetime VALUE time::now() READONLY;
DEFINE FIELD avatar ON user TYPE option<string>;

DEFINE FIELD permissions ON user TYPE array<record<permission>> 
    DEFAULT [permission:create_post, permission:create_comment];

DEFINE INDEX unique_username ON user COLUMNS username UNIQUE;
DEFINE INDEX unique_email ON user COLUMNS email UNIQUE;

DEFINE SCOPE user_scope
    SESSION 30d
    SIGNUP (
        CREATE user
        SET
            username = $username,
            email = $email,
            avatar = "https://www.gravatar.com/avatar/" + crypto::md5($email),
            password = crypto::argon2::generate($password)
    )
    SIGNIN (
        SELECT *
        FROM user
        WHERE username = $username AND crypto::argon2::compare(password, $password)
    );