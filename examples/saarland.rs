use gv100ad::{Ags, Database};

fn main() {
    let db = Database::from_path("GV100AD_300421.txt").unwrap();

    let ags_land = Ags::new_land(10);

    let land = db.get_land(&ags_land).unwrap();
    println!("{}:", land.name);

    for kreis in db.iter_kreise_in(&ags_land) {
        println!("  {}:", kreis.name);

        for gemeinde in db.iter_gemeinden_in(&kreis.ags) {
            println!("    {}: {} residents", gemeinde.name, gemeinde.population_total,);
        }
    }
}
