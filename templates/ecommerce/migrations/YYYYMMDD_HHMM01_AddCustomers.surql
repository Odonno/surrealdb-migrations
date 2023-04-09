INSERT INTO address {
    id: 'pratim_home', 
    number: '221B', 
    street: 'Baker street', 
    city: 'London', 
    country: 'UK'
};

INSERT INTO customer {
    id: 'pratim',
    name: 'Pratim',
    email: 'abc@gmail.com',
    address: address:pratim_home
};

INSERT INTO address {
    id: 'tobie_home', 
    number: '221A', 
    street: 'Church street', 
    city: 'London', 
    country: 'UK'
};

INSERT INTO customer {
    id: 'tobie',
    name: 'Tobie',
    email: 'tobie@gmail.com',
    address: address:tobie_home
};

INSERT INTO address {
    id: 'alex_home', 
    number: '221C', 
    street: 'Pound street', 
    city: 'London', 
    country: 'UK'
};

INSERT INTO customer {
    id: 'alex',
    name: 'Alex',
    email: 'alex@gmail.com',
    address: address:alex_home
};