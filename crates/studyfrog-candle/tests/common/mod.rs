//! The labelled answers used to calibrate the runtimes.

use Class::{OffTopic, Paraphrase, Partial, WrongFact};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Class {
    Paraphrase,
    Partial,
    OffTopic,
    WrongFact,
}

/// (reference, answer, class): eleven facts, four answers each.
pub const SET: [(&str, &str, Class); 44] = [
    (
        "The mitochondria is the powerhouse of the cell",
        "it produces most of the cell's energy",
        Paraphrase,
    ),
    (
        "The mitochondria is the powerhouse of the cell",
        "the mitochondria is part of the cell",
        Partial,
    ),
    (
        "The mitochondria is the powerhouse of the cell",
        "the capital of France is Paris",
        OffTopic,
    ),
    (
        "The mitochondria is the powerhouse of the cell",
        "the nucleus is the powerhouse of the cell",
        WrongFact,
    ),
    (
        "Paris is the capital of France",
        "France's capital city is Paris",
        Paraphrase,
    ),
    (
        "Paris is the capital of France",
        "Paris is a big city",
        Partial,
    ),
    (
        "Paris is the capital of France",
        "photosynthesis converts light to energy",
        OffTopic,
    ),
    (
        "Paris is the capital of France",
        "Lyon is the capital of France",
        WrongFact,
    ),
    (
        "Water boils at 100 degrees Celsius at sea level",
        "at sea level water reaches its boiling point at 100 C",
        Paraphrase,
    ),
    (
        "Water boils at 100 degrees Celsius at sea level",
        "water gets hot when heated",
        Partial,
    ),
    (
        "Water boils at 100 degrees Celsius at sea level",
        "the heart pumps blood around the body",
        OffTopic,
    ),
    (
        "Water boils at 100 degrees Celsius at sea level",
        "water boils at 50 degrees Celsius at sea level",
        WrongFact,
    ),
    (
        "Photosynthesis converts light into chemical energy",
        "plants turn sunlight into stored sugar energy",
        Paraphrase,
    ),
    (
        "Photosynthesis converts light into chemical energy",
        "plants need light",
        Partial,
    ),
    (
        "Photosynthesis converts light into chemical energy",
        "the treaty of Versailles ended the war",
        OffTopic,
    ),
    (
        "Photosynthesis converts light into chemical energy",
        "photosynthesis converts chemical energy into light",
        WrongFact,
    ),
    (
        "The heart pumps blood around the body",
        "it moves blood through the body",
        Paraphrase,
    ),
    (
        "The heart pumps blood around the body",
        "the heart is an organ",
        Partial,
    ),
    (
        "The heart pumps blood around the body",
        "bananas are yellow",
        OffTopic,
    ),
    (
        "The heart pumps blood around the body",
        "the lungs pump blood around the body",
        WrongFact,
    ),
    (
        "The French Revolution began in 1789",
        "the revolution in France started in 1789",
        Paraphrase,
    ),
    (
        "The French Revolution began in 1789",
        "there was a revolution in France",
        Partial,
    ),
    (
        "The French Revolution began in 1789",
        "sodium is a soft metal",
        OffTopic,
    ),
    (
        "The French Revolution began in 1789",
        "the French Revolution began in 1889",
        WrongFact,
    ),
    (
        "Gravity pulls objects towards the centre of the earth",
        "objects are attracted towards the earth's centre by gravity",
        Paraphrase,
    ),
    (
        "Gravity pulls objects towards the centre of the earth",
        "gravity is a force",
        Partial,
    ),
    (
        "Gravity pulls objects towards the centre of the earth",
        "the opera was sung in Italian",
        OffTopic,
    ),
    (
        "Gravity pulls objects towards the centre of the earth",
        "gravity pushes objects away from the earth",
        WrongFact,
    ),
    (
        "A noun is a word that names a person, place or thing",
        "nouns are words for people, places and things",
        Paraphrase,
    ),
    (
        "A noun is a word that names a person, place or thing",
        "a noun is a kind of word",
        Partial,
    ),
    (
        "A noun is a word that names a person, place or thing",
        "the river flooded in spring",
        OffTopic,
    ),
    (
        "A noun is a word that names a person, place or thing",
        "a verb is a word that names an action",
        WrongFact,
    ),
    (
        "Insulin lowers the level of sugar in the blood",
        "the hormone insulin brings blood glucose down",
        Paraphrase,
    ),
    (
        "Insulin lowers the level of sugar in the blood",
        "insulin is a hormone",
        Partial,
    ),
    (
        "Insulin lowers the level of sugar in the blood",
        "the train leaves at noon",
        OffTopic,
    ),
    (
        "Insulin lowers the level of sugar in the blood",
        "insulin raises the level of sugar in the blood",
        WrongFact,
    ),
    (
        "Supply and demand determine the price of goods",
        "prices of goods are set by how much is offered and wanted",
        Paraphrase,
    ),
    (
        "Supply and demand determine the price of goods",
        "prices matter in a shop",
        Partial,
    ),
    (
        "Supply and demand determine the price of goods",
        "the cat sat on the mat",
        OffTopic,
    ),
    (
        "Supply and demand determine the price of goods",
        "the weather determines the price of goods",
        WrongFact,
    ),
    (
        "The Pythagorean theorem relates the sides of a right triangle",
        "in a right triangle the squares of the two short sides add up to the square of the long side",
        Paraphrase,
    ),
    (
        "The Pythagorean theorem relates the sides of a right triangle",
        "it is about triangles",
        Partial,
    ),
    (
        "The Pythagorean theorem relates the sides of a right triangle",
        "enzymes speed up reactions",
        OffTopic,
    ),
    (
        "The Pythagorean theorem relates the sides of a right triangle",
        "the Pythagorean theorem relates the sides of a circle",
        WrongFact,
    ),
];

pub fn verdict(v: f32) -> Class {
    if v >= 0.7 {
        Paraphrase
    } else if v >= 0.5 {
        Partial
    } else {
        OffTopic
    }
}
