use regex::Regex;
use std::fs::{self, File};
use std::io::{self, Write};



fn main() -> io::Result<()> {
    // Read the schema.rs file
    println!("Reading schema.rs file...");
    let schema_content = fs::read_to_string("../webserver/src/schema.rs")?;

    //println!("Schema content: '{}'", schema_content);
    //let schema_stripped = schema_content.split("\n").collect::<Vec<&str>>().join(" ");
    //println!("Schema stripped: '{:?}'", schema_stripped);

    let re = r#"
        table!\s*
        \{\s*
            (\w+)\s*
                \(([^\)]+)\)\s*
            \{\s*
                ([^}]+)\s*
            \}\s*
        \}\s*
    "#.replace(" ", "").replace("\n", "");

    //println!("re: '{}'", re);
    
    // Regular expression to match table definitions
    let table_re = Regex::new(re.as_str()).unwrap();
    // Regular expression to match column definitions within a table
    let column_re = Regex::new(r"(\w+)\s*->\s*(\w+(<\w+>)?)").unwrap();

    // Open the up.sql file for writing
    println!("Creating up.sql file...");
    let mut up_sql = match File::create("../webserver/migrations/up.sql/up.sql")  {
        Ok(file) => file,
        Err(e) => {
            println!("ERROR: Failed to create up.sql file: {}", e);

            return Err(e);
        }
    };

    // Iterate over each table definition
    println!("Iterate over each table definition..");
    let mut number_of_tables = 0;
    for table_match in table_re.captures_iter(&schema_content) {
        let table_name = &table_match[1];
        let primary_key = &table_match[2];
        let columns_definition = &table_match[3];

        // Start the CREATE TABLE statement
        println!("Creating table: '{}' with primary_key '{}'", table_name, primary_key);
        writeln!(up_sql, "CREATE TABLE IF NOT EXISTS {} (", table_name)?;

        // Iterate over each column definition
        println!("  Iterate over each column definition..");
        println!("     columns_definition: '{}'", columns_definition);
        let mut column_iter = column_re.captures_iter(columns_definition);
        let mut primary_key_found = false;
        let mut first_column_name = "".to_string(); 
        
        if let Some(first_column) = column_iter.next() {
            if first_column.len() < 2   {
                panic!("(1) Column definition first_column not found for table: '{}'", table_name);
            } else {
                let mut id_or_mandatory = "NOT NULL";
                first_column_name = first_column[1].to_string();
                if first_column_name == primary_key {
                    id_or_mandatory = "PRIMARY KEY";
                    primary_key_found = true;
                } else {
                    println!("WARNING: First column '{}' is not  primary key: '{}'", first_column_name, primary_key);
                }
                println!("      first_column: '{}' ==> '{}' {}", first_column[1].to_string(), first_column_name, id_or_mandatory);                
                write!(up_sql, "    {} {} {}", first_column_name, map_column_type(&first_column[2]).to_string(), id_or_mandatory)?;

                // Alernative way
                //   ALTER TABLE table ADD COLUMN IF NOT EXISTS column column_type;
                // To drop a column use 
                //   ALTER TABLE table DROP COLUMN IF EXISTS column
                // To find the columns in the table use
                //   SELECT * FROM information_schema.columns WHERE table_name = 'table_name';

            }
        }
        
        // Add the remaining columns
        for column_match in column_iter {
            if column_match.len() < 2 {
                panic!("(2) Column definition column_match not found for table: {}", table_name);
            } else {
                let mut id_or_mandatory = "NOT NULL";
                let column_name = column_match[1].to_string();                
                if column_name == primary_key {
                    id_or_mandatory = "PRIMARY KEY";
                    primary_key_found = true;
                    println!("WARNING: Primary key '{}' is not in first column '{}' ", primary_key, first_column_name);
                }
                println!("      column_match: '{}' => '{}' {}", column_match[0].to_string(), column_name, id_or_mandatory);
                write!(up_sql, ",\n    {} {} {}", column_name, map_column_type(&column_match[2]).to_string(), id_or_mandatory)?;
            }
        }
                
        // Add the primary key constraint if not already present
        if !primary_key_found {
            println!("    Add the primary key constraint '{}'...", primary_key);
            write!(up_sql, ",\n    PRIMARY KEY ({})", primary_key)?;
        }
        writeln!(up_sql, "\n);")?;
        number_of_tables += 1; 
    }

    if number_of_tables == 0 {
        panic!("ERROR: No table found in schema.rs file.");
    } else {
        println!("up.sql generated successfully .");
    }
    Ok(())
}

// Function to map Diesel types to SQL types
fn map_column_type(diesel_type: &str) -> &str {
    match diesel_type {
        "Uuid" => "UUID",
        "Varchar" => "VARCHAR",
        "Int4" => "INT",
        "Int2" => "SMALLINT",
        "Int8" => "BIGINT",
        "Text" => "TEXT",
        "Bool" => "BOOLEAN",
        "Float4" => "REAL",
        "Float8" => "DOUBLE PRECISION",
        "Timestamp" => "TIMESTAMP",
        "Date" => "DATE",
        "Time" => "TIME",
        "Interval" => "INTERVAL",
        "DateTime" => "DATE",
        "Bytea" => "BYTEA",
        "Array<Text>" => "TEXT[]", // Assuming TEXT[] for Array<Text>
        "Array<Uuid>" => "UUID[]",
        "Array<Int8>" => "BIGINT[]",
        "Array<Int4>" => "INT[]",
        "Array<Int2>" => "SMALLINT[]",
        "Array<Bool>" => "BOOLEAN[]",
        _ => "TEXT", // Default to TEXT for unknown types
    }
}

