use gv100ad::{
    model::{
        gemeinde::GemeindeDaten,
        kreis::KreisDaten,
        land::{LandDaten, LandSchluessel},
    },
    Database,
};

fn main() {
    let db = Database::from_path("GV100AD3004/GV100AD_300421.txt").unwrap();

    let schluessel = "10".parse::<LandSchluessel>().unwrap();
    let land = db.get::<_, LandDaten>(schluessel).unwrap();

    println!("{}:", land.name);

    for (_, kreis) in db.children::<_, KreisDaten>(schluessel) {
        println!("  {}:", kreis.name);

        for (_, gemeinde) in db.children::<_, GemeindeDaten>(kreis.schluessel) {
            println!(
                "    {}: {} residents",
                gemeinde.name, gemeinde.population_total
            );
        }
    }
}
