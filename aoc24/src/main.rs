use std::cell::Cell;
use std::error::Error;
use std::io::{self, Write};
use std::result;

type Result<T> = result::Result<T, Box<Error>>;

fn main() -> Result<()> {
    let combat = Combat {
        army1: Army::test1_immune(),
        army2: Army::test1_infection(),
    };
    let winner = combat.fight_to_end();
    writeln!(
        io::stdout(),
        "test: {} wins with {} units left",
        winner.name,
        winner.total_live_units(),
    )?;

    let combat = Combat {
        army1: Army::real_immune(),
        army2: Army::real_infection(),
    };
    let winner = combat.fight_to_end();
    writeln!(
        io::stdout(),
        "real: {} wins with {} units left",
        winner.name,
        winner.total_live_units(),
    )?;

    let mut combat = Combat {
        army1: Army::test1_immune(),
        army2: Army::test1_infection(),
    };
    combat.army1.boost(1570);
    let winner = combat.fight_to_end();
    writeln!(
        io::stdout(),
        "test: {} wins with {} units left after {} boost",
        winner.name,
        winner.total_live_units(),
        1570,
    )?;

    // Trying this with boost values of 40 or 41 results in a combat that
    // does not appear to terminate. I just kept trying higher values until
    // I saw the first combat that terminated. 42 really is apparently the
    // ultimate answer to the ultimate question of life, the Universe and
    // Everything.
    for boost in 42.. {
        let mut combat = Combat {
            army1: Army::real_immune(),
            army2: Army::real_infection(),
        };
        combat.army1.boost(boost);
        let winner = combat.fight_to_end();
        if winner.name == "immune" {
            writeln!(
                io::stdout(),
                "real: {} wins with {} units left after {} boost",
                winner.name,
                winner.total_live_units(),
                boost,
            )?;
            return Ok(());
        } else if boost % 1 == 0 {
            writeln!(
                io::stdout(),
                "real: {} wins with {} units left after {} boost",
                winner.name,
                winner.total_live_units(),
                boost,
            )?;
        }
    }
    Err(From::from("no minimal boost could be found"))
}

#[derive(Clone, Debug)]
struct Combat {
    army1: Army,
    army2: Army,
}

#[derive(Clone, Debug)]
struct Plan<'g> {
    attacker: &'g Group,
    victim: &'g Group,
}

#[derive(Clone, Debug)]
struct Army {
    name: String,
    groups: Vec<Group>,
}

#[derive(Clone, Debug)]
struct Group {
    army: String,
    id: u64,
    units: Cell<u64>,
    unit_hp: u64,
    initiative: u64,
    attack: Attack,
    weaknesses: Vec<AttackKind>,
    immunities: Vec<AttackKind>,
}

#[derive(Clone, Debug)]
struct Attack {
    kind: AttackKind,
    damage: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum AttackKind {
    Radiation,
    Cold,
    Fire,
    Slashing,
    Bludgeoning,
}

impl Combat {
    fn fight_to_end(&self) -> &Army {
        loop {
            if let Some(winner) = self.fight() {
                return winner;
            }
        }
    }

    fn fight(&self) -> Option<&Army> {
        for plan in self.target_selection() {
            if !plan.attacker.is_alive() {
                continue;
            }

            let damage = plan.attacker.attack_damage(plan.victim);
            plan.victim.absorb(damage);
        }
        self.winner()
    }

    fn winner(&self) -> Option<&Army> {
        assert!(self.army1.is_alive() || self.army2.is_alive());
        if !self.army1.is_alive() {
            Some(&self.army2)
        } else if !self.army2.is_alive() {
            Some(&self.army1)
        } else {
            None
        }
    }

    fn target_selection(&self) -> Vec<Plan> {
        let mut plans = self.army1.target_selection(&self.army2);
        plans.extend(self.army2.target_selection(&self.army1));
        plans.sort_by(|plan1, plan2| {
            plan1.attacker.initiative.cmp(&plan2.attacker.initiative).reverse()
        });
        plans
    }
}

impl Army {
    fn is_alive(&self) -> bool {
        self.groups.iter().any(|g| g.is_alive())
    }

    fn total_live_units(&self) -> u64 {
        self.groups.iter().map(|g| g.units.get()).sum()
    }

    fn target_selection<'a>(&'a self, enemy: &'a Army) -> Vec<Plan<'a>> {
        let mut plans = vec![];
        let mut candidates: Vec<&Group> = enemy.alive_groups();
        for g in self.target_selection_order() {
            if let Some(i) = g.choose_victim(&candidates) {
                plans.push(Plan { attacker: g, victim: candidates[i] });
                candidates.swap_remove(i);
            }
        }
        plans
    }

    fn target_selection_order(&self) -> Vec<&Group> {
        let mut groups = self.alive_groups();
        groups.sort_by(|g1, g2| {
            let power1 = g1.effective_power();
            let power2 = g2.effective_power();
            if power1 != power2 {
                power1.cmp(&power2).reverse()
            } else {
                g1.initiative.cmp(&g2.initiative).reverse()
            }
        });
        groups
    }

    fn alive_groups(&self) -> Vec<&Group> {
        self.groups.iter().filter(|g| g.is_alive()).collect()
    }

    fn boost(&mut self, amount: u64) {
        for g in self.groups.iter_mut() {
            g.attack.damage += amount;
        }
    }
}

impl Group {
    fn is_alive(&self) -> bool {
        self.units.get() > 0
    }

    fn effective_power(&self) -> u64 {
        self.units.get() * self.attack.damage
    }

    fn absorb(&self, damage: u64) -> u64 {
        let units_lost = damage / self.unit_hp;
        let old = self.units.get();
        self.units.set(old.saturating_sub(units_lost));
        old - self.units.get()
    }

    fn choose_victim(&self, candidates: &[&Group]) -> Option<usize> {
        let mut choice = None;
        for (i, &candidate) in candidates.iter().enumerate() {
            let damage = self.attack_damage(candidate);
            if damage == 0 {
                continue;
            }
            if choice.is_none() {
                choice = Some(i);
                continue;
            }
            let cur = choice.unwrap();
            let cur_damage = self.attack_damage(candidates[cur]);
            if damage < cur_damage {
                continue;
            } else if damage > cur_damage {
                choice = Some(i);
                continue;
            }

            let epower = candidate.effective_power();
            let cur_epower = candidates[cur].effective_power();
            if epower < cur_epower {
                continue;
            } else if epower > cur_epower {
                choice = Some(i);
                continue;
            }

            let init = candidate.initiative;
            let cur_init = candidates[cur].initiative;
            assert!(init != cur_init);
            if init > cur_init {
                choice = Some(i);
            }
        }
        choice
    }

    fn attack_damage(&self, group: &Group) -> u64 {
        if group.is_immune(&self.attack.kind) {
            return 0;
        }
        let mut damage = self.effective_power();
        if group.is_weak(&self.attack.kind) {
            damage *= 2;
        }
        damage
    }

    fn is_immune(&self, kind: &AttackKind) -> bool {
        self.immunities.contains(kind)
    }

    fn is_weak(&self, kind: &AttackKind) -> bool {
        self.weaknesses.contains(kind)
    }
}

impl Attack {
    fn new(kind: AttackKind, damage: u64) -> Attack {
        Attack { kind, damage }
    }
}

// Below is just the construction of inputs. Didn't feel like writing a parser.

impl Army {
    fn test1_immune() -> Army {
        use self::AttackKind::*;

        Army {
            name: "immune".to_string(),
            groups: vec![
                Group {
                    army: "immune".to_string(),
                    id: 1,
                    units: Cell::new(17),
                    unit_hp: 5390,
                    initiative: 2,
                    attack: Attack::new(Fire, 4507),
                    weaknesses: vec![Radiation, Bludgeoning],
                    immunities: vec![],
                },
                Group {
                    army: "immune".to_string(),
                    id: 2,
                    units: Cell::new(989),
                    unit_hp: 1274,
                    initiative: 3,
                    attack: Attack::new(Slashing, 25),
                    weaknesses: vec![Bludgeoning, Slashing],
                    immunities: vec![Fire],
                },
            ],
        }
    }

    fn test1_infection() -> Army {
        use self::AttackKind::*;

        Army {
            name: "infection".to_string(),
            groups: vec![
                Group {
                    army: "infection".to_string(),
                    id: 1,
                    units: Cell::new(801),
                    unit_hp: 4706,
                    initiative: 1,
                    attack: Attack::new(Bludgeoning, 116),
                    weaknesses: vec![Radiation],
                    immunities: vec![],
                },
                Group {
                    army: "infection".to_string(),
                    id: 2,
                    units: Cell::new(4485),
                    unit_hp: 2961,
                    initiative: 4,
                    attack: Attack::new(Slashing, 12),
                    weaknesses: vec![Fire, Cold],
                    immunities: vec![Radiation],
                },
            ],
        }
    }

    fn real_immune() -> Army {
        use self::AttackKind::*;

        Army {
            name: "immune".to_string(),
            groups: vec![
                Group {
                    army: "immune".to_string(),
                    id: 1,
                    units: Cell::new(479),
                    unit_hp: 3393,
                    initiative: 8,
                    attack: Attack::new(Cold, 66),
                    weaknesses: vec![Radiation],
                    immunities: vec![],
                },
                Group {
                    army: "immune".to_string(),
                    id: 2,
                    units: Cell::new(2202),
                    unit_hp: 4950,
                    initiative: 2,
                    attack: Attack::new(Cold, 18),
                    weaknesses: vec![Fire],
                    immunities: vec![Slashing],
                },
                Group {
                    army: "immune".to_string(),
                    id: 3,
                    units: Cell::new(8132),
                    unit_hp: 9680,
                    initiative: 7,
                    attack: Attack::new(Radiation, 9),
                    weaknesses: vec![Bludgeoning, Fire],
                    immunities: vec![Slashing],
                },
                Group {
                    army: "immune".to_string(),
                    id: 4,
                    units: Cell::new(389),
                    unit_hp: 13983,
                    initiative: 13,
                    attack: Attack::new(Cold, 256),
                    weaknesses: vec![],
                    immunities: vec![Bludgeoning],
                },
                Group {
                    army: "immune".to_string(),
                    id: 5,
                    units: Cell::new(1827),
                    unit_hp: 5107,
                    initiative: 18,
                    attack: Attack::new(Slashing, 24),
                    weaknesses: vec![],
                    immunities: vec![],
                },
                Group {
                    army: "immune".to_string(),
                    id: 6,
                    units: Cell::new(7019),
                    unit_hp: 2261,
                    initiative: 16,
                    attack: Attack::new(Fire, 3),
                    weaknesses: vec![],
                    immunities: vec![Radiation, Slashing, Cold],
                },
                Group {
                    army: "immune".to_string(),
                    id: 7,
                    units: Cell::new(4736),
                    unit_hp: 8421,
                    initiative: 3,
                    attack: Attack::new(Slashing, 17),
                    weaknesses: vec![Cold],
                    immunities: vec![],
                },
                Group {
                    army: "immune".to_string(),
                    id: 8,
                    units: Cell::new(491),
                    unit_hp: 3518,
                    initiative: 1,
                    attack: Attack::new(Radiation, 65),
                    weaknesses: vec![Cold],
                    immunities: vec![Fire, Bludgeoning],
                },
                Group {
                    army: "immune".to_string(),
                    id: 9,
                    units: Cell::new(2309),
                    unit_hp: 7353,
                    initiative: 20,
                    attack: Attack::new(Bludgeoning, 31),
                    weaknesses: vec![],
                    immunities: vec![Radiation],
                },
                Group {
                    army: "immune".to_string(),
                    id: 10,
                    units: Cell::new(411),
                    unit_hp: 6375,
                    initiative: 14,
                    attack: Attack::new(Bludgeoning, 151),
                    weaknesses: vec![Cold, Fire],
                    immunities: vec![Slashing],
                },
            ],
        }
    }

    fn real_infection() -> Army {
        use self::AttackKind::*;

        Army {
            name: "infection".to_string(),
            groups: vec![
                Group {
                    army: "infection".to_string(),
                    id: 1,
                    units: Cell::new(148),
                    unit_hp: 31914,
                    initiative: 4,
                    attack: Attack::new(Cold, 416),
                    weaknesses: vec![Bludgeoning],
                    immunities: vec![Radiation, Cold, Fire],
                },
                Group {
                    army: "infection".to_string(),
                    id: 2,
                    units: Cell::new(864),
                    unit_hp: 38189,
                    initiative: 6,
                    attack: Attack::new(Slashing, 72),
                    weaknesses: vec![],
                    immunities: vec![],
                },
                Group {
                    army: "infection".to_string(),
                    id: 3,
                    units: Cell::new(2981),
                    unit_hp: 7774,
                    initiative: 15,
                    attack: Attack::new(Fire, 4),
                    weaknesses: vec![],
                    immunities: vec![Bludgeoning, Cold],
                },
                Group {
                    army: "infection".to_string(),
                    id: 4,
                    units: Cell::new(5259),
                    unit_hp: 22892,
                    initiative: 5,
                    attack: Attack::new(Fire, 8),
                    weaknesses: vec![],
                    immunities: vec![],
                },
                Group {
                    army: "infection".to_string(),
                    id: 5,
                    units: Cell::new(318),
                    unit_hp: 16979,
                    initiative: 9,
                    attack: Attack::new(Bludgeoning, 106),
                    weaknesses: vec![Fire],
                    immunities: vec![],
                },
                Group {
                    army: "infection".to_string(),
                    id: 6,
                    units: Cell::new(5017),
                    unit_hp: 32175,
                    initiative: 17,
                    attack: Attack::new(Bludgeoning, 11),
                    weaknesses: vec![Slashing],
                    immunities: vec![Radiation],
                },
                Group {
                    army: "infection".to_string(),
                    id: 7,
                    units: Cell::new(4308),
                    unit_hp: 14994,
                    initiative: 10,
                    attack: Attack::new(Fire, 5),
                    weaknesses: vec![Slashing],
                    immunities: vec![Fire, Cold],
                },
                Group {
                    army: "infection".to_string(),
                    id: 8,
                    units: Cell::new(208),
                    unit_hp: 14322,
                    initiative: 19,
                    attack: Attack::new(Cold, 133),
                    weaknesses: vec![Radiation],
                    immunities: vec![],
                },
                Group {
                    army: "infection".to_string(),
                    id: 9,
                    units: Cell::new(3999),
                    unit_hp: 48994,
                    initiative: 11,
                    attack: Attack::new(Cold, 20),
                    weaknesses: vec![Cold, Slashing],
                    immunities: vec![],
                },
                Group {
                    army: "infection".to_string(),
                    id: 10,
                    units: Cell::new(1922),
                    unit_hp: 34406,
                    initiative: 12,
                    attack: Attack::new(Slashing, 35),
                    weaknesses: vec![Slashing],
                    immunities: vec![],
                },
            ],
        }
    }
}
