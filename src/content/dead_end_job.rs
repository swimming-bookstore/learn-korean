use super::{v, vh, Item, Lesson};

const fn lesson(
    episode: u16,
    korean: &'static str,
    meaning: &'static str,
    vocab: &'static [Item],
    grammar: &'static [Item],
) -> Lesson {
    Lesson {
        series_n: 3,
        episode,
        series: "막다른 일자리",
        korean,
        meaning,
        vocab,
        grammar,
    }
}

/// Lines heard in the Dead-End Job trailer. A single recognition pass drops
/// or garbles speech after about 20 seconds, so these lines come from
/// windows shorter than that.
///
/// Parse standard, the same split as 참교육 and the true-education Khaiii pass.
/// Vocabulary is the content stem, in sentence order, in dictionary form.
/// A verb or adjective ends in 다: 시작하다, not 시작하고요. Particles and
/// endings are not vocabulary. One English reading, no slash. 의 is "of",
/// not "'s", because A의 B is "B of A".
///
/// Grammar is the written eojeol: that dictionary stem, then the particle or
/// ending. 거미인력 is the company name Spider Staffing, so the name stays
/// one word and is not parsed into spider plus manpower.
pub const LESSONS: &[Lesson] = &[
    lesson(
        0,
        "나 이번 생은 망한 것 같지",
        "I think this life is ruined.",
        &[
            v("나", "I"),
            v("이번", "this time"),
            vh("생", "生", "a life"),
            vh("망하다", "亡", "to be ruined"),
            v("것", "a thing"),
            v("같다", "to seem"),
        ],
        &[
            v("생은", "생 + 은 (life, topic)"),
            v("망한", "망하다 + ㄴ (that is ruined)"),
            v("같지", "같다 + 지 (I think)"),
        ],
    ),
    lesson(
        1,
        "마지막 반전의 기회",
        "One last chance to turn it around.",
        &[
            v("마지막", "last"),
            vh("반전", "反轉", "a reversal"),
            vh("기회", "機會", "a chance"),
        ],
        &[v("반전의", "반전 + 의 (of a reversal)")],
    ),
    lesson(
        2,
        "거미인력",
        "Spider Staffing.",
        &[v("거미인력", "Spider Staffing")],
        &[],
    ),
    lesson(
        3,
        "일자리 알아보러 오셨어요?",
        "Did you come to look for work?",
        &[
            v("일자리", "a job"),
            v("알아보다", "to look into"),
            v("오다", "to come"),
        ],
        &[
            v("알아보러", "알아보다 + 러 (in order to look)"),
            v("오셨어요", "오다 + 시 + 었 + 어요 (you came)"),
        ],
    ),
    lesson(
        4,
        "최저 시급의 기본 10배부터 시작하고요",
        "And it starts from ten times the base minimum wage.",
        &[
            vh("최저", "最低", "the lowest"),
            vh("시급", "時給", "hourly wage"),
            vh("기본", "基本", "base"),
            v("10", "ten"),
            vh("배", "倍", "times"),
            vh("시작하다", "始作", "to start"),
        ],
        &[
            v("시급의", "시급 + 의 (of the wage)"),
            v("10배부터", "10 + 배 + 부터 (from ten times)"),
            v("시작하고요", "시작하다 + 고 + 요 (starts, and)"),
        ],
    ),
    lesson(
        5,
        "하루 일하고 하루 일당으로 바로 드립니다",
        "Work a day, and we pay a day's wage right away.",
        &[
            v("하루", "one day"),
            v("일하다", "to work"),
            vh("일당", "日當", "a day's pay"),
            v("바로", "right away"),
            v("드리다", "to give"),
        ],
        &[
            v("일하고", "일하다 + 고 (work, and)"),
            v("일당으로", "일당 + 으로 (as a day's pay)"),
            v("드립니다", "드리다 + ㅂ니다 (we give)"),
        ],
    ),
    lesson(
        6,
        "하루 뜨고 오세요",
        "Come for a day.",
        &[v("하루", "one day"), v("뜨다", "to show up"), v("오다", "to come")],
        &[
            v("뜨고", "뜨다 + 고 (show up, and)"),
            v("오세요", "오다 + 세요 (please come)"),
        ],
    ),
    lesson(
        7,
        "돈 벌어야죠",
        "You've got to make money.",
        &[v("돈", "money"), v("벌다", "to earn")],
        &[v("벌어야죠", "벌다 + 어야죠 (you've got to earn)")],
    ),
    lesson(
        8,
        "알바를 성공하시면",
        "If you succeed at the side job.",
        &[v("알바", "a side job"), vh("성공하다", "成功", "to succeed")],
        &[
            v("알바를", "알바 + 를 (the side job, object)"),
            v("성공하시면", "성공하다 + 시 + 면 (if you succeed)"),
        ],
    ),
    lesson(
        9,
        "일당이 최소 두 배 높아집니다",
        "The day's pay goes up at least double.",
        &[
            vh("일당", "日當", "a day's pay"),
            vh("최소", "最小", "at least"),
            v("두", "two"),
            vh("배", "倍", "times"),
            v("높아지다", "to get higher"),
        ],
        &[
            v("일당이", "일당 + 이 (the pay, subject)"),
            v("높아집니다", "높아지다 + ㅂ니다 (goes up)"),
        ],
    ),
    lesson(
        10,
        "이 알바가 졸라 불길해",
        "This side job feels really wrong.",
        &[
            v("이", "this"),
            v("알바", "a side job"),
            v("졸라", "really"),
            vh("불길하다", "不吉", "ominous"),
        ],
        &[
            v("알바가", "알바 + 가 (the gig, subject)"),
            v("불길해", "불길하다 + 어 (feels ominous)"),
        ],
    ),
    lesson(
        11,
        "저주예요",
        "It's a curse.",
        &[vh("저주", "詛呪", "a curse"), v("이다", "to be")],
        &[v("저주예요", "저주 + 이다 + 예요 (it is a curse)")],
    ),
    lesson(
        12,
        "살려! 살려!",
        "Help! Help!",
        &[v("살리다", "to save")],
        &[v("살려", "살리다 + 어 (save me)")],
    ),
    lesson(
        13,
        "이거 다 뭐예요?",
        "What was all of that?",
        &[v("이거", "this"), v("다", "all"), v("뭐", "what"), v("이다", "to be")],
        &[v("뭐예요", "뭐 + 이다 + 예요 (what is it)")],
    ),
    lesson(
        14,
        "환상이에요?",
        "Is it a hallucination?",
        &[vh("환상", "幻想", "a hallucination"), v("이다", "to be")],
        &[v("환상이에요", "환상 + 이다 + 이에요 (is it a hallucination)")],
    ),
    lesson(
        15,
        "귀신, 괴물",
        "Ghosts. Monsters.",
        &[vh("귀신", "鬼神", "ghost"), vh("괴물", "怪物", "monster")],
        &[],
    ),
    lesson(
        16,
        "귀신 무섭냐? 그지가 더 무섭지",
        "Scared of ghosts? Being broke is scarier.",
        &[
            vh("귀신", "鬼神", "ghost"),
            v("무섭다", "to be scary"),
            v("그지", "broke"),
            v("더", "more"),
        ],
        &[
            v("무섭냐", "무섭다 + 냐 (are you scared)"),
            v("그지가", "그지 + 가 (being broke, subject)"),
            v("무섭지", "무섭다 + 지 (is scarier)"),
        ],
    ),
    lesson(
        17,
        "꿀알바 시작하겠습니다",
        "Starting the sweet gig.",
        &[v("꿀알바", "a sweet gig"), vh("시작하다", "始作", "to start")],
        &[v("시작하겠습니다", "시작하다 + 겠 + 습니다 (I will start)")],
    ),
    lesson(
        18,
        "여덟 시간만 버티면 됩니다",
        "Just hold on for eight hours.",
        &[
            v("여덟", "eight"),
            vh("시간", "時間", "hours"),
            v("버티다", "to hold on"),
            v("되다", "to become"),
        ],
        &[
            v("시간만", "시간 + 만 (just the hours)"),
            v("버티면", "버티다 + 면 (if you hold on)"),
            v("됩니다", "되다 + ㅂ니다 (that's enough)"),
        ],
    ),
    lesson(
        19,
        "안 그러면 여기서 못 나갈 수도 있어",
        "If you don't, you may not get out of here.",
        &[
            v("안", "not"),
            v("그렇다", "to be so"),
            v("여기", "here"),
            v("못", "cannot"),
            v("나가다", "to get out"),
            v("수", "a way"),
            v("있다", "to exist"),
        ],
        &[
            v("그러면", "그렇다 + 면 (if so)"),
            v("여기서", "여기 + 서 (from here)"),
            v("나갈", "나가다 + ㄹ (to get out)"),
            v("수도", "수 + 도 (might)"),
            v("있어", "있다 + 어 (there is)"),
        ],
    ),
    lesson(
        20,
        "할 수 있다",
        "I can do it.",
        &[v("하다", "to do"), v("수", "a way"), v("있다", "to exist")],
        &[v("할 수 있다", "하다 + ㄹ 수 있다 (can)")],
    ),
    lesson(
        21,
        "궁금하지 않아요?",
        "Aren't you curious?",
        &[v("궁금하다", "to be curious"), v("않다", "to not")],
        &[
            v("궁금하지", "궁금하다 + 지 (curious, and)"),
            v("않아요", "않다 + 아요 (not, polite)"),
        ],
    ),
    lesson(
        22,
        "거미인력이 뭐 하는 곳인지?",
        "What kind of place Spider Staffing is.",
        &[
            v("거미인력", "Spider Staffing"),
            v("뭐", "what"),
            v("하다", "to do"),
            v("곳", "a place"),
            v("이다", "to be"),
        ],
        &[
            v("거미인력이", "거미인력 + 이 (Spider Staffing, subject)"),
            v("하는", "하다 + 는 (that does)"),
            v("곳인지", "곳 + 이다 + ㄴ지 (whether it is a place)"),
        ],
    ),
    lesson(
        23,
        "혹시 여기서 일했었나요?",
        "Did he work here, by any chance?",
        &[v("혹시", "by any chance"), v("여기", "here"), v("일하다", "to work")],
        &[
            v("여기서", "여기 + 서 (here)"),
            v("일했었나요", "일하다 + 었었 + 나요 (did you work)"),
        ],
    ),
    lesson(
        24,
        "문을 세우라고",
        "Stop. I said stop.",
        &[vh("문", "門", "door"), v("세우다", "to stop")],
        &[
            v("문을", "문 + 을 (the door, object)"),
            v("세우라고", "세우다 + 라고 (saying to stop)"),
        ],
    ),
    lesson(
        25,
        "일하라고",
        "Work.",
        &[v("일하다", "to work")],
        &[v("일하라고", "일하다 + 라고 (saying to work)")],
    ),
    lesson(
        26,
        "혁준 씨, 어디까지 갈 수 있나 한번 해보시죠",
        "Hyeok-jun, let's see how far you can go.",
        &[
            v("혁준", "Hyeok-jun"),
            v("씨", "honorific"),
            v("어디", "where"),
            v("가다", "to go"),
            v("수", "a way"),
            v("있다", "to exist"),
            v("한번", "one try"),
            v("해보다", "to try"),
        ],
        &[
            v("어디까지", "어디 + 까지 (how far)"),
            v("갈 수 있나", "가다 + ㄹ 수 있다 + 나 (whether one can go)"),
            v("해보시죠", "해보다 + 시 + 죠 (please try)"),
        ],
    ),
    lesson(
        27,
        "악몽이라뇨. 알바죠, 꿀알바",
        "A nightmare? Please. A side job. A sweet one.",
        &[
            vh("악몽", "惡夢", "a nightmare"),
            v("이다", "to be"),
            v("알바", "a side job"),
            v("꿀알바", "a sweet gig"),
        ],
        &[
            v("악몽이라뇨", "악몽 + 이다 + 라뇨 (what do you mean, a nightmare)"),
            v("알바죠", "알바 + 죠 (it's a side job)"),
        ],
    ),
    lesson(
        28,
        "뭘 알바?",
        "What side job?",
        &[v("뭐", "what"), v("알바", "a side job")],
        &[v("뭘", "뭐 + ㄹ (what, object)")],
    ),
];
