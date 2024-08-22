use unicode_segmentation::UnicodeSegmentation;

pub struct NewSubscriber {
    pub email: String,
    pub name: SubscriberName,
}

pub struct SubscriberName(String);

impl SubscriberName {
    pub fn inner(self) -> String {
        // 호출자는 inner 문자열을 얻는다.
        // 하지만 내부 스트링에는 더 이상 SubscriberName이 존재하지 않는다.
        // 'inner'는 'self'를 값으로 받고,
        // move 구문에 따라 그것을 소비하기 때문이다.
        self.0
    }
    // 가변 참조자를 노출한다.
    pub fn inner_mut(&mut self) -> &mut str {
        // 호출자는 inner 문자열에 대한 가변 참조자를 얻는다.
        // 호출자는 임의로 그 값을 변경할 수 있으며,
        // 이는 잠재적으로 불변량을 깨뜨릴 수 있다.
        &mut self.0
    }
    pub fn inner_ref(&self) -> &str {
        // 호출자는 inner 문자열에 대한 공유 참조자를 얻는다.
        // 호출자는 읽기 전용으로 접근할 수 있으며,
        // 이는 불변량을 깨뜨리지 못한다.
        &self.0
    }
    /// 입력이 subscriber 이름에 대한 검증 조건을 모두 만족하면
    /// `SubscriberName` 인스턴스를 반환한다.
    /// 그렇지 않으면 패닉에 빠진다.
    pub fn parse(s: String) -> SubscriberName {
        // `.trim()`은 입력 `s`에 대해 뒤에 이어지는 공백 같은 문자를 제외한 뷰를 반환한다.
        // `.is_emty`는 해당 뷰가 문자를 포함하고 있는지 확인한다.
        let is_empty_or_whitespace = s.trim().is_empty();

        let is_tool_long = s.graphemes(true).count() > 256;

        let forbidden_characters = ['/', '(', ')', '<', '>', '\\', '{', '}'];
        let contains_forbidden_characters = s
            .chars()
            .any(|g| forbidden_characters.contains(&g));

        if is_empty_or_whitespace || is_tool_long || contains_forbidden_characters {
            panic!("{} is not a valid subscriber name", s)
        } else {
            Self(s)
        }
    }
}