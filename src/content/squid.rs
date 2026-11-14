use super::{v, vh, Item, Lesson};

const fn lesson(
    episode: u16,
    korean: &'static str,
    meaning: &'static str,
    vocab: &'static [Item],
    grammar: &'static [Item],
) -> Lesson {
    Lesson {
        series_n: 4,
        episode,
        series: "오징어 게임",
        korean,
        meaning,
        vocab,
        grammar,
    }
}

/// Lines heard in the Squid Game official trailer. A single recognition pass
/// smears the opening across the music and garbles 빚을 지고, so these lines
/// come from windows shorter than 20 seconds.
///
/// Parse standard, the same split as 참교육 and the Dead-End Job lesson.
/// Vocabulary is the content stem, in sentence order, in dictionary form.
/// A verb or adjective ends in 다: 피다, not 피었습니다. Particles and
/// endings are not vocabulary. One English reading, no slash. 의 is "of",
/// not "'s", because A의 B is "B of A".
///
/// Grammar is the written eojeol: that dictionary stem, then the particle or
/// ending. 무궁화 is the hibiscus of the children's game, so the name stays
/// one word.
pub const LESSONS: &[Lesson] = &[
    lesson(
        0,
        "여러분들은 모두 감당할 수 없는 빚을 지고",
        "You are all carrying debts you cannot bear.",
        &[
            v("여러분", "you all"),
            v("모두", "all"),
            v("감당하다", "to bear"),
            v("수", "a way"),
            v("없다", "to not exist"),
            v("빚", "a debt"),
            v("지다", "to carry"),
        ],
        &[
            v("여러분들은", "여러분 + 들 + 은 (you all, topic)"),
            v("감당할 수 없는", "감당하다 + ㄹ 수 없다 + 는 (that you cannot bear)"),
            v("빚을", "빚 + 을 (the debt, object)"),
            v("지고", "지다 + 고 (carrying, and)"),
        ],
    ),
    lesson(
        1,
        "벼랑 끝에 서 계신 분들입니다",
        "You are people standing at the edge of a cliff.",
        &[
            v("벼랑", "a cliff"),
            v("끝", "an edge"),
            v("서다", "to stand"),
            v("계시다", "to be"),
            v("분", "a person"),
            v("이다", "to be"),
        ],
        &[
            v("끝에", "끝 + 에 (at the edge)"),
            v("서 계신", "서다 + 계시다 + ㄴ (who are standing)"),
            v("분들입니다", "분 + 들 + 이다 + ㅂ니다 (are people)"),
        ],
    ),
    lesson(
        2,
        "게임의 참가를 원하지 않는 분은",
        "Anyone who does not want to join the game.",
        &[
            v("게임", "a game"),
            vh("참가", "參加", "joining"),
            v("원하다", "to want"),
            v("않다", "to not do"),
            v("분", "a person"),
        ],
        &[
            v("게임의", "게임 + 의 (of the game)"),
            v("참가를", "참가 + 를 (joining, object)"),
            v("원하지 않는", "원하다 + 지 않다 + 는 (who does not want)"),
            v("분은", "분 + 은 (a person, topic)"),
        ],
    ),
    lesson(
        3,
        "지금 말씀해 주시길 바랍니다",
        "Please say so now.",
        &[
            v("지금", "now"),
            v("말씀하다", "to say"),
            v("주시다", "to give"),
            v("바라다", "to hope"),
        ],
        &[
            v("말씀해", "말씀하다 + 어 (say)"),
            v("주시길", "주시다 + 기 + 를 (that you give)"),
            v("바랍니다", "바라다 + ㅂ니다 (we hope)"),
        ],
    ),
    lesson(
        4,
        "무궁화 꽃이 피었습니다",
        "The hibiscus flower has bloomed.",
        &[
            vh("무궁화", "無窮花", "hibiscus"),
            v("꽃", "a flower"),
            v("피다", "to bloom"),
        ],
        &[
            v("꽃이", "꽃 + 이 (the flower, subject)"),
            v("피었습니다", "피다 + 었 + 습니다 (has bloomed)"),
        ],
    ),
    lesson(
        5,
        "그 옛날에 어릴 때 하던 거 말이야?",
        "You mean the one we played as kids?",
        &[
            v("그", "that"),
            v("옛날", "the old days"),
            v("어리다", "to be young"),
            v("때", "a time"),
            v("하다", "to do"),
            v("거", "a thing"),
            v("말", "a word"),
        ],
        &[
            v("옛날에", "옛날 + 에 (in the old days)"),
            v("어릴 때", "어리다 + ㄹ 때 (when one was young)"),
            v("하던", "하다 + 던 (that we used to do)"),
            v("말이야", "말 + 이다 + 야 (you mean)"),
        ],
    ),
    lesson(
        6,
        "이 사람을 그렇게 죽여 놓고",
        "After killing this person like that.",
        &[
            v("이", "this"),
            v("사람", "a person"),
            v("그렇게", "like that"),
            v("죽이다", "to kill"),
            v("놓다", "to leave"),
        ],
        &[
            v("사람을", "사람 + 을 (the person, object)"),
            v("죽여", "죽이다 + 어 (kill)"),
            v("놓고", "놓다 + 고 (and leave it)"),
        ],
    ),
    lesson(
        7,
        "여섯 개의 게임을 모두 이긴 분들께는",
        "To those who win all six games.",
        &[
            v("여섯", "six"),
            v("개", "a piece"),
            v("게임", "a game"),
            v("모두", "all"),
            v("이기다", "to win"),
            v("분", "a person"),
        ],
        &[
            v("개의", "개 + 의 (of pieces)"),
            v("게임을", "게임 + 을 (the games, object)"),
            v("이긴", "이기다 + ㄴ (who won)"),
            v("분들께는", "분 + 들 + 께 + 는 (to those people)"),
        ],
    ),
    lesson(
        8,
        "456억 원의 상금이 지급됩니다",
        "A prize of 45.6 billion won will be paid.",
        &[
            vh("456억", "億", "45.6 billion"),
            vh("원", "圓", "won"),
            vh("상금", "賞金", "a prize"),
            vh("지급되다", "支給", "to be paid"),
        ],
        &[
            v("원의", "원 + 의 (of won)"),
            v("상금이", "상금 + 이 (the prize, subject)"),
            v("지급됩니다", "지급되다 + ㅂ니다 (is paid)"),
        ],
    ),
    lesson(
        9,
        "눈 바짝 뜨고 계세요",
        "Keep your eyes wide open.",
        &[
            v("눈", "an eye"),
            v("바짝", "wide"),
            v("뜨다", "to open"),
            v("계시다", "to stay"),
        ],
        &[
            v("뜨고", "뜨다 + 고 (open, and)"),
            v("계세요", "계시다 + 세요 (please stay)"),
        ],
    ),
    lesson(
        10,
        "제일 센 쪽에 붙어",
        "Stick to the strongest side.",
        &[
            vh("제일", "第一", "the most"),
            v("세다", "to be strong"),
            v("쪽", "a side"),
            v("붙다", "to stick"),
        ],
        &[
            v("센", "세다 + ㄴ (that is strong)"),
            v("쪽에", "쪽 + 에 (to the side)"),
            v("붙어", "붙다 + 어 (stick)"),
        ],
    ),
    lesson(
        11,
        "그게 살 길이야",
        "That is how you live.",
        &[
            v("그게", "that"),
            vh("살", "生", "life"),
            v("길", "a way"),
            v("이다", "to be"),
        ],
        &[v("살 길이야", "살 + 길 + 이다 + 야 (is the way to live)")],
    ),
    lesson(
        12,
        "애들 게임 규칙 단순해",
        "A kids' game. The rules are simple.",
        &[
            v("애들", "kids"),
            v("게임", "a game"),
            vh("규칙", "規則", "a rule"),
            vh("단순하다", "單純", "to be simple"),
        ],
        &[v("단순해", "단순하다 + 어 (is simple)")],
    ),
    lesson(
        13,
        "내가 무조건 이기게 해 줄게",
        "I will make sure you win.",
        &[
            v("내", "I"),
            vh("무조건", "無條件", "no matter what"),
            v("이기다", "to win"),
            v("하다", "to do"),
            v("주다", "to give"),
        ],
        &[
            v("내가", "내 + 가 (I, subject)"),
            v("이기게", "이기다 + 게 (so that you win)"),
            v("해 줄게", "하다 + 어 주다 + ㄹ게 (I will do it for you)"),
        ],
    ),
    lesson(
        14,
        "무슨 수로?",
        "By what means?",
        &[v("무슨", "what"), v("수", "a means")],
        &[v("수로", "수 + 로 (by means)")],
    ),
    lesson(
        15,
        "내가 앞을 봐 줄게",
        "I will watch the front for you.",
        &[
            v("내", "I"),
            v("앞", "the front"),
            v("보다", "to look"),
            v("주다", "to give"),
        ],
        &[
            v("내가", "내 + 가 (I, subject)"),
            v("앞을", "앞 + 을 (the front, object)"),
            v("봐 줄게", "보다 + 아 주다 + ㄹ게 (I will look for you)"),
        ],
    ),
    lesson(
        16,
        "이러면 안 되는 거잖아",
        "We are not supposed to do this.",
        &[
            v("이러다", "to do this"),
            v("안", "not"),
            v("되다", "to be allowed"),
            v("거", "a thing"),
        ],
        &[
            v("이러면", "이러다 + 면 (if we do this)"),
            v("안 되는", "되다 + 는 (that is not allowed)"),
            v("거잖아", "거 + 이다 + 잖아 (you know it is)"),
        ],
    ),
    lesson(
        17,
        "우리끼리 이렇게 죽이면 안 되는 거잖아",
        "We are not supposed to kill each other like this.",
        &[
            v("우리", "we"),
            v("끼리", "among"),
            v("이렇게", "like this"),
            v("죽이다", "to kill"),
            v("안", "not"),
            v("되다", "to be allowed"),
            v("거", "a thing"),
        ],
        &[
            v("우리끼리", "우리 + 끼리 (among us)"),
            v("죽이면", "죽이다 + 면 (if we kill)"),
            v("안 되는", "되다 + 는 (that is not allowed)"),
            v("거잖아", "거 + 이다 + 잖아 (you know it is)"),
        ],
    ),
    lesson(
        18,
        "게임을 거부하는 참가자는 탈락으로 처리된다",
        "A player who refuses the game is treated as eliminated.",
        &[
            v("게임", "a game"),
            vh("거부하다", "拒否", "to refuse"),
            vh("참가자", "參加者", "a player"),
            vh("탈락", "脫落", "elimination"),
            vh("처리되다", "處理", "to be treated"),
        ],
        &[
            v("게임을", "게임 + 을 (the game, object)"),
            v("거부하는", "거부하다 + 는 (who refuses)"),
            v("참가자는", "참가자 + 는 (a player, topic)"),
            v("탈락으로", "탈락 + 으로 (as elimination)"),
            v("처리된다", "처리되다 + ㄴ다 (is treated)"),
        ],
    ),
];
