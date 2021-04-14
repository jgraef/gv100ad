use gv100ad::{
    model::{
        gemeinde::GemeindeDaten,
        kreis::KreisDaten,
        land::{LandDaten, LandSchluessel},
    },
    Database,
};

fn main() {
    // Open the database. Refer to the `README.md` file for the source of the datasets.
    let db = Database::from_path("GV100AD3004/GV100AD_300421.txt").unwrap();

    // Parse a key for the state of Saarland
    let schluessel = "10".parse::<LandSchluessel>().unwrap();

    // Get the record for the state of Saarland
    let land = db.get::<_, LandDaten>(schluessel).unwrap();
    println!("{}:", land.name);

    // Iterate over the districts (Kreise) in Saarland
    for kreis in db.children::<_, KreisDaten>(schluessel) {
        println!("  {}:", kreis.name);

        // Iterate over the municipalities (Gemeinde) in the district (Kreis)
        for gemeinde in db.children::<_, GemeindeDaten>(kreis.schluessel) {
            println!(
                "    {}: {} residents",
                gemeinde.name, gemeinde.population_total
            );
        }
    }

    // Get the sum of the population of all municipalities in Saarland
    let total_population: u64 = db.children::<_, GemeindeDaten>(schluessel)
        .map(|gemeinde| gemeinde.population_total)
        .sum();
    println!("Total population of {}: {}", land.name, total_population);
}
