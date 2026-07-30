use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Category {
    pub id: u32,
    pub name: &'static str,
}

pub const CATEGORIES: &[Category] = &[
    Category {
        id: 6000,
        name: "Business",
    },
    Category {
        id: 6001,
        name: "Weather",
    },
    Category {
        id: 6002,
        name: "Utilities",
    },
    Category {
        id: 6003,
        name: "Travel",
    },
    Category {
        id: 6004,
        name: "Sports",
    },
    Category {
        id: 6005,
        name: "Social Networking",
    },
    Category {
        id: 6006,
        name: "Reference",
    },
    Category {
        id: 6007,
        name: "Productivity",
    },
    Category {
        id: 6008,
        name: "Photo & Video",
    },
    Category {
        id: 6009,
        name: "News",
    },
    Category {
        id: 6010,
        name: "Navigation",
    },
    Category {
        id: 6011,
        name: "Music",
    },
    Category {
        id: 6012,
        name: "Lifestyle",
    },
    Category {
        id: 6013,
        name: "Health & Fitness",
    },
    Category {
        id: 6014,
        name: "Games",
    },
    Category {
        id: 6015,
        name: "Finance",
    },
    Category {
        id: 6016,
        name: "Entertainment",
    },
    Category {
        id: 6017,
        name: "Education",
    },
    Category {
        id: 6018,
        name: "Books",
    },
    Category {
        id: 6020,
        name: "Medical",
    },
    Category {
        id: 6021,
        name: "Magazines & Newspapers",
    },
    Category {
        id: 6022,
        name: "Catalogs",
    },
    Category {
        id: 6023,
        name: "Food & Drink",
    },
    Category {
        id: 6024,
        name: "Shopping",
    },
    Category {
        id: 6025,
        name: "Stickers",
    },
    Category {
        id: 6026,
        name: "Developer Tools",
    },
    Category {
        id: 6027,
        name: "Graphics & Design",
    },
    Category {
        id: 7001,
        name: "Games: Action",
    },
    Category {
        id: 7002,
        name: "Games: Adventure",
    },
    Category {
        id: 7003,
        name: "Games: Casual",
    },
    Category {
        id: 7004,
        name: "Games: Board",
    },
    Category {
        id: 7005,
        name: "Games: Card",
    },
    Category {
        id: 7006,
        name: "Games: Casino",
    },
    Category {
        id: 7009,
        name: "Games: Family",
    },
    Category {
        id: 7011,
        name: "Games: Music",
    },
    Category {
        id: 7012,
        name: "Games: Puzzle",
    },
    Category {
        id: 7013,
        name: "Games: Racing",
    },
    Category {
        id: 7014,
        name: "Games: Role Playing",
    },
    Category {
        id: 7015,
        name: "Games: Simulation",
    },
    Category {
        id: 7016,
        name: "Games: Sports",
    },
    Category {
        id: 7017,
        name: "Games: Strategy",
    },
    Category {
        id: 7018,
        name: "Games: Trivia",
    },
    Category {
        id: 7019,
        name: "Games: Word",
    },
];

pub fn is_valid_category(id: u32) -> bool {
    CATEGORIES.iter().any(|category| category.id == id)
}
