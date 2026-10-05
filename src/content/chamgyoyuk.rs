use super::{v, Item, Lesson};

const fn lesson(
    episode: u16,
    korean: &'static str,
    meaning: &'static str,
    vocab: &'static [Item],
    grammar: &'static [Item],
) -> Lesson {
    Lesson {
        series_n: 2,
        episode,
        series: "참교육",
        korean,
        meaning,
        vocab,
        grammar,
    }
}

pub const LESSONS: &[Lesson] = &[
    lesson(
        0,
        "본 작품은 가상으로 만들어진 허구의 이야기입니다",
        "This work is a fictional story created as fiction.",
        &[
            v("본 本", "this / the present"),
            v("작품 作品", "work (of art)"),
            v("가상 假想", "fiction / imagination"),
            v("만들다", "to make"),
            v("허구 虛構", "fiction"),
            v("이야기", "story"),
        ],
        &[
            v("작품은", "작품 + 은 (topic)"),
            v("가상으로", "가상 + 으로 (as / by means of)"),
            v("만들어진", "만들 + 어지다 + ㄴ (that was made)"),
            v("허구의", "허구 + 의 (of fiction)"),
            v("이야기입니다", "이야기 + 이다 + ㅂ니다 (is a story)"),
        ],
    ),
    lesson(
        1,
        "2011년 체벌금지법의 통과 이후",
        "After the 2011 ban on corporal punishment passed.",
        &[
            v("2011년 年", "the year 2011"),
            v("체벌 體罰", "corporal punishment"),
            v("금지법 禁止法", "a ban / prohibition law"),
            v("통과 通過", "passage / passing"),
            v("이후 以後", "after / since"),
        ],
        &[
            v("2011년", "2011 + 년 (year)"),
            v("체벌금지법의", "체벌금지법 + 의 (of the ban)"),
            v("통과 이후", "after the passage"),
        ],
    ),
];
