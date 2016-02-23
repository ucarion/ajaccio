use std::num::Wrapping;

use bitboard::Bitboard;
use square::Square;

pub struct MagicDatabase {
    rook_databases: Vec<Vec<Bitboard>>,
    bishop_databases: Vec<Vec<Bitboard>>,

    rook_attacks: Vec<Bitboard>,
    bishop_attacks: Vec<Bitboard>
}

impl MagicDatabase {
    pub fn new() -> MagicDatabase {
        let mut db = MagicDatabase {
            rook_databases: Vec::new(),
            bishop_databases: Vec::new(),
            rook_attacks: Vec::new(),
            bishop_attacks: Vec::new()
        };

        for square_index in 0..64 {
            let (magic, shift_amount) = ROOK_MAGICS[square_index];
            let database = gen_rook_database(square_index as u8, magic, shift_amount).unwrap();
            db.rook_databases.push(database);

            db.rook_attacks.push(rook_attacks(Square::new(square_index as u8)));
        }

        for square_index in 0..64 {
            let (magic, shift_amount) = BISHOP_MAGICS[square_index];
            let database = gen_bishop_database(square_index as u8, magic, shift_amount).unwrap();
            db.bishop_databases.push(database);

            db.bishop_attacks.push(bishop_attacks(Square::new(square_index as u8)));
        }

        db
    }

    pub fn rook_attacks(&self, square: Square, occupied: Bitboard) -> Bitboard {
        let square_index = square.to_index() as usize;

        let variation = occupied & self.rook_attacks[square_index];
        let (magic, shift) = ROOK_MAGICS[square_index];
        let magic_index = magic_index(magic, shift, variation);

        self.rook_databases[square_index][magic_index]
    }

    pub fn bishop_attacks(&self, square: Square, occupied: Bitboard) -> Bitboard {
        let square_index = square.to_index() as usize;

        let variation = occupied & self.bishop_attacks[square_index];
        let (magic, shift) = BISHOP_MAGICS[square_index];
        let magic_index = magic_index(magic, shift, variation);

        self.bishop_databases[square_index][magic_index]
    }

    pub fn queen_attacks(&self, square: Square, occupied: Bitboard) -> Bitboard {
        self.rook_attacks(square, occupied) | self.bishop_attacks(square, occupied)
    }
}

fn magic_index(magic: u64, shift_amount: u32, bitboard: Bitboard) -> usize {
    let hash = Wrapping(magic) * Wrapping(bitboard.0);
    (hash.0 >> (64 - shift_amount)) as usize
}

fn gen_rook_database(square_index: u8, magic: u64, shift_amount: u32) -> Option<Vec<Bitboard>> {
    gen_database(&rook_attacks, &rook_move_locations, square_index, magic, shift_amount)
}

fn gen_bishop_database(square_index: u8, magic: u64, shift_amount: u32) -> Option<Vec<Bitboard>> {
    gen_database(&bishop_attacks, &bishop_move_locations, square_index, magic, shift_amount)
}

fn gen_database(attacks: &Fn(Square) -> Bitboard,
                solver: &Fn(Square, Bitboard) -> Bitboard,
                square_index: u8, magic: u64, shift_amount: u32) -> Option<Vec<Bitboard>> {
    let square = Square::new(square_index);
    let attacks = attacks(square);
    let num_bits = attacks.num_occupied_squares();
    let variations = gen_variations(attacks);

    assert_eq!(num_bits, shift_amount);

    gen_magic_database(magic, num_bits, square, &variations, &solver)
}

pub fn find_rook_magic(square_index: u8) -> (u64, u32) {
    find_magic(&rook_attacks, &rook_move_locations, square_index)
}

pub fn find_bishop_magic(square_index: u8) -> (u64, u32) {
    find_magic(&bishop_attacks, &bishop_move_locations, square_index)
}

fn find_magic(attacks: &Fn(Square) -> Bitboard,
              solver: &Fn(Square, Bitboard) -> Bitboard,
              square_index: u8) -> (u64, u32) {
    let square = Square::new(square_index);
    let attacks = attacks(square);
    let num_bits = attacks.num_occupied_squares();
    let variations = gen_variations(attacks);

    loop {
        use rand::random;

        let magic = random::<u64>() & random::<u64>() & random::<u64>() & random::<u64>();
        let db = gen_magic_database(magic, num_bits, square, &variations, &solver);

        if db.is_some() {
            return (magic, num_bits);
        }
    }
}

fn gen_variations(bitboard: Bitboard) -> Vec<Bitboard> {
    if bitboard.is_empty() {
        return vec![bitboard];
    }

    let top_one_square = Square::new(63 - bitboard.0.leading_zeros() as u8);
    let without_top = bitboard ^ top_one_square.to_bitboard();
    let rest_variations = gen_variations(without_top);

    let with_one: Vec<_> = rest_variations.iter()
        .map(|bitboard| bitboard.clone() | top_one_square.to_bitboard())
        .collect();
    let without_one = rest_variations;

    let mut out = Vec::new();
    out.extend(with_one);
    out.extend(without_one);

    out
}

fn gen_magic_database(magic: u64, num_bits: u32, square: Square,
             variations: &[Bitboard],
             solver: &Fn(Square, Bitboard) -> Bitboard) -> Option<Vec<Bitboard>> {
    let database_size = 2usize.pow(num_bits);
    let mut database = vec![Bitboard::new(0); database_size];

    for variation in variations {
        let index = magic_index(magic, num_bits, variation.clone());
        let solution = solver(square, variation.clone());

        if database[index].is_empty() {
            database[index] = solution;
        } else if database[index] != solution {
            return None;
        }
    }

    Some(database)
}

pub fn rook_attacks(square: Square) -> Bitboard {
    let mut result = Bitboard::new(0);

    for rank in (square.rank() + 1)..7 {
        result = result | Square::from_coords(square.file(), rank).to_bitboard();
    }

    for rank in 1..square.rank() {
        result = result | Square::from_coords(square.file(), rank).to_bitboard();
    }

    for file in (square.file() + 1)..7 {
        result = result | Square::from_coords(file, square.rank()).to_bitboard();
    }

    for file in 1..square.file() {
        result = result | Square::from_coords(file, square.rank()).to_bitboard();
    }

    result
}

pub fn rook_move_locations(square: Square, enemies: Bitboard) -> Bitboard {
    let mut result = Bitboard::new(0);

    for rank in (square.rank() + 1)..8 {
        let square = Square::from_coords(square.file(), rank).to_bitboard();
        result = result | square;

        if (square & enemies).is_nonempty() {
            break;
        }
    }

    for rank in (0..square.rank()).rev() {
        let square = Square::from_coords(square.file(), rank).to_bitboard();
        result = result | square;

        if (square & enemies).is_nonempty() {
            break;
        }
    }

    for file in (square.file() + 1)..8 {
        let square = Square::from_coords(file, square.rank()).to_bitboard();
        result = result | square;

        if (square & enemies).is_nonempty() {
            break;
        }
    }

    for file in (0..square.file()).rev() {
        let square = Square::from_coords(file, square.rank()).to_bitboard();
        result = result | square;

        if (square & enemies).is_nonempty() {
            break;
        }
    }

    result
}

pub fn bishop_attacks(square: Square) -> Bitboard {
    let start = (square.file() as i8, square.rank() as i8);

    diagonal_attacks(start, 1, 1) |
        diagonal_attacks(start, 1, -1) |
        diagonal_attacks(start, -1, 1) |
        diagonal_attacks(start, -1, -1)
}

fn diagonal_attacks(start: (i8, i8), dx: i8, dy: i8) -> Bitboard {
    fn is_in_bounds(file: i8, rank: i8) -> bool {
        1 <= file && file <= 6 && 1 <= rank && rank <= 6
    }

    let mut result = Bitboard::new(0);
    let mut cursor = start;
    loop {
        cursor = (cursor.0 + dx, cursor.1 + dy);

        if is_in_bounds(cursor.0, cursor.1) {
            result = result | Square::from_coords(cursor.0 as u8, cursor.1 as u8).to_bitboard();
        } else {
            break;
        }
    }

    result
}

pub fn bishop_move_locations(square: Square, enemies: Bitboard) -> Bitboard {
    let start = (square.file() as i8, square.rank() as i8);

    diagonal_move_locations(start, 1, 1, enemies) |
        diagonal_move_locations(start, 1, -1, enemies) |
        diagonal_move_locations(start, -1, 1, enemies) |
        diagonal_move_locations(start, -1, -1, enemies)
}

fn diagonal_move_locations(start: (i8, i8), dx: i8, dy: i8, enemies: Bitboard) -> Bitboard {
    fn is_in_bounds(file: i8, rank: i8) -> bool {
        0 <= file && file < 8 && 0 <= rank && rank < 8
    }

    fn occupied_by_enemy(file: i8, rank: i8, enemies: Bitboard) -> bool {
        let square = Square::from_coords(file as u8, rank as u8);
        (square.to_bitboard() & enemies).is_nonempty()
    }

    let mut result = Bitboard::new(0);
    let mut cursor = start;
    loop {
        cursor = (cursor.0 + dx, cursor.1 + dy);

        if is_in_bounds(cursor.0, cursor.1) {
            result = result | Square::from_coords(cursor.0 as u8, cursor.1 as u8).to_bitboard();

            if occupied_by_enemy(cursor.0, cursor.1, enemies) {
                break;
            }
        } else {
            break;
        }
    }

    result
}

#[test]
fn test_magic_database() {
    let occupied = Bitboard::new(4521262379438080);
    let square = Square::from_san("b6");

    let rook_expected = Bitboard(144710032489971712);
    let bishop_expected = Bitboard(577868148796030976);
    let queen_expected = Bitboard(722578181286002688);

    let database = MagicDatabase::new();
    assert_eq!(rook_expected, database.rook_attacks(square, occupied));
    assert_eq!(bishop_expected, database.bishop_attacks(square, occupied));
    assert_eq!(queen_expected, database.queen_attacks(square, occupied));
}

#[test]
fn test_rook_attacks() {
    // TODO: Is there a better way to establish that I want these cases to work than to hard-code
    // the correct values?
    let a1 = Bitboard::new(282578800148862);
    let e4 = Bitboard::new(4521262379438080);
    let h8 = Bitboard::new(9115426935197958144);

    assert_eq!(a1, rook_attacks(Square::from_san("a1")));
    assert_eq!(e4, rook_attacks(Square::from_san("e4")));
    assert_eq!(h8, rook_attacks(Square::from_san("h8")));
}

#[test]
fn test_bishop_attacks() {
    let a1 = Bitboard::new(18049651735527936);
    let e4 = Bitboard::new(637888545440768);
    let h8 = Bitboard::new(18049651735527936);

    assert_eq!(a1, bishop_attacks(Square::from_san("a1")));
    assert_eq!(e4, bishop_attacks(Square::from_san("e4")));
    assert_eq!(h8, bishop_attacks(Square::from_san("h8")));
}

#[test]
fn test_bishop_move_locations() {
    let enemies = Bitboard::new(4521262379438080);
    let expected = Bitboard(1227793891648880768);
    assert_eq!(expected, bishop_move_locations(Square::from_san("c6"), enemies));
}

#[test]
fn test_rook_move_locations() {
    let enemies = Bitboard::new(4521262379438080);
    let expected = Bitboard(289385980119482368);
    assert_eq!(expected, rook_move_locations(Square::from_san("c6"), enemies));
}

const BISHOP_MAGICS: [(u64, u32); 64] = [
    (13528393349890082, 6),
    (9152340191895557, 5),
    (3459899212118884352, 5),
    (1165484472926210, 5),
    (73206171372101698, 5),
    (4611844400178267136, 5),
    (1130315167301632, 5),
    (39586723008512, 6),
    (72092920211120192, 5),
    (9009407270723712, 5),
    (2269396865134593, 5),
    (18159826110513152, 5),
    (2207881789696, 5),
    (585468510176542720, 5),
    (5764682987826323456, 5),
    (4614089551936751680, 5),
    (1214136051040768, 5),
    (22518032530179072, 5),
    (845533166045824, 7),
    (76701965546962944, 7),
    (1128101088067720, 7),
    (562960825649152, 7),
    (571750350865408, 5),
    (3463831063946725504, 5),
    (633662832394752, 5),
    (322158115963904, 5),
    (4543182079788032, 7),
    (184651999952769056, 9),
    (2959885310369792, 9),
    (144717857920421888, 7),
    (1130297958663168, 5),
    (2341951280341189184, 5),
    (2306159737282498560, 5),
    (285941744795904, 5),
    (1729452831947620416, 7),
    (144401078279471632, 9),
    (162130690391945280, 9),
    (141845592080544, 7),
    (4627450270685628416, 5),
    (288797725225452608, 5),
    (1153495587719487552, 5),
    (11404152320033280, 5),
    (9512165707765797890, 7),
    (412719513856, 7),
    (1020484380524672, 7),
    (283708393259328, 7),
    (585478955302134272, 5),
    (1301544692579565600, 5),
    (37155830831907074, 5),
    (72603402371072, 5),
    (283682623848464, 5),
    (4400739778560, 5),
    (216177249418874880, 5),
    (4616260055560388608, 5),
    (2308103609549717568, 5),
    (4612257773089062928, 5),
    (3941785827279364, 6),
    (288231836474216579, 5),
    (18014403374944264, 5),
    (70368746309632, 5),
    (2594073394494324992, 5),
    (10376294177653915904, 5),
    (2449980256309231744, 5),
    (9009415609983488, 6)
];

const ROOK_MAGICS: [(u64, u32); 64] = [
    (180166250207477760, 12),
    (18014708284002304, 11),
    (72092778548494352, 11),
    (9295464883984859140, 11),
    (144123992760914961, 11),
    (36050787276687872, 11),
    (1225050567001244036, 11),
    (72060070120661248, 12),
    (6917669775334178944, 11),
    (36169809395712128, 10),
    (2305983815429939200, 10),
    (2305983781062836352, 10),
    (578853323973608448, 10),
    (288371130820067456, 10),
    (141287277741312, 10),
    (281483570872576, 11),
    (9007751158054912, 11),
    (74451231925346560, 10),
    (141287781109768, 10),
    (141287378391040, 10),
    (2306125583970992196, 10),
    (4612812470237790720, 10),
    (72198881315661056, 10),
    (9805486477952682113, 11),
    (140741785436416, 11),
    (9886810712850432, 10),
    (9223794429709647936, 10),
    (17594341918720, 10),
    (36072788221755520, 10),
    (4616191819225759872, 10),
    (1153202983878656004, 10),
    (4620693501151281537, 11),
    (36028934537609280, 11),
    (35186595086336, 10),
    (144396732862570496, 10),
    (36037595267870722, 10),
    (18058381130466304, 10),
    (563121785672720, 10),
    (4504733565854224, 10),
    (3458768914048090177, 11),
    (140876001345536, 11),
    (74591143714684964, 10),
    (1161084547399808, 10),
    (1729399849230565504, 10),
    (8796915171332, 10),
    (19316237725598016, 10),
    (288511859718422540, 10),
    (11529365680545726468, 11),
    (36028934462111808, 11),
    (1170936180679642176, 10),
    (144194490368790784, 10),
    (8813541360256, 10),
    (145241122350900608, 10),
    (4398080098432, 10),
    (4611967510617522944, 10),
    (564050606459392, 11),
    (140814934262018, 12),
    (4612037872886808993, 11),
    (9259454710480373825, 11),
    (1153203014020894721, 11),
    (562984851112962, 11),
    (281526516843009, 11),
    (288283154991612420, 11),
    (13194684809474, 12)
];
