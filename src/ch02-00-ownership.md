# 2장: 소유권 시스템

## Rust의 가장 독특한 개념

소유권(Ownership)은 Rust가 다른 모든 언어와 구별되는 핵심 개념입니다. 이것을 이해하지 못하면 Rust 코드를 작성할 수 없습니다. 반대로 이것을 제대로 이해하면, Rust 코드의 90%가 자연스럽게 풀립니다.

처음에는 컴파일러가 당신의 코드를 계속 거부하는 것처럼 느껴집니다. 이건 정상입니다. Rust를 만든 사람들도, 수십 년 경력의 시스템 프로그래머들도 처음에는 같은 경험을 합니다.

## 왜 소유권이 필요한가?

### 메모리 관리의 세 가지 방법

프로그램은 실행 중에 메모리를 사용하고, 더 이상 필요 없으면 해제해야 합니다. 역사적으로 세 가지 방법이 있었습니다:

**1. 수동 메모리 관리 (C, C++)**

```c
// C 코드
char* create_message() {
    char* msg = malloc(256);  // 개발자가 직접 할당
    strcpy(msg, "Hello");
    return msg;
}

int main() {
    char* m = create_message();
    printf("%s\n", m);
    free(m);  // 개발자가 직접 해제 — 잊으면 메모리 누수!
    // free 후에 m을 사용하면? use-after-free 버그!
    // free를 두 번 하면? double-free 버그!
}
```

장점: 빠르다, GC 없음
단점: 매우 위험하다. 역사상 대부분의 보안 취약점이 여기서 나왔음

**2. 가비지 컬렉터 (Java, Go, JavaScript, Python)**

```javascript
// JavaScript 코드
function createMessage() {
    let msg = "Hello";  // 힙에 할당
    return msg;
}

let m = createMessage();
console.log(m);
// m이 더 이상 참조되지 않으면 GC가 알아서 해제
// 개발자는 신경 쓸 필요 없음
```

장점: 안전하다, 편리하다
단점: GC 실행 시 프로그램이 멈춤(stop-the-world), 메모리 사용량이 많음, 실시간 시스템에 부적합

**3. 소유권 시스템 (Rust)**

```rust
// Rust 코드
fn create_message() -> String {
    let msg = String::from("Hello");  // 힙에 할당
    msg  // 소유권이 호출자에게 이동
}   // msg가 여기서 drop — 하지만 소유권이 이미 이동했으므로 해제 안 됨

fn main() {
    let m = create_message();  // 소유권 획득
    println!("{}", m);
}   // m이 여기서 drop — 자동으로 메모리 해제
```

장점: 안전하다(컴파일 타임에 검증), GC 없다(예측 가능한 성능)
단점: 배우기 어렵다

### 블록체인에서 왜 중요한가?

스마트 컨트랙트는 한 번 배포하면 수정이 불가능합니다. 메모리 버그가 있는 컨트랙트는 해킹당해도 되돌릴 수 없습니다.

- **C/C++로 만든 시스템**: 메모리 버그 가능, 높은 성능
- **Java/Go로 만든 시스템**: 안전하지만 GC로 인한 예측 불가 레이턴시
- **Rust로 만든 시스템**: 메모리 안전 + GC 없는 예측 가능한 성능

이것이 Solana, Near, Polkadot이 Rust를 선택한 핵심 이유입니다.

## 소유권이 해결하는 문제들

소유권 시스템은 다음 문제들을 컴파일 타임에 방지합니다:

### 1. Use-After-Free (해제 후 사용)

```c
// C에서 발생하는 버그
int* ptr = malloc(sizeof(int));
*ptr = 42;
free(ptr);      // 메모리 해제
printf("%d\n", *ptr);  // 버그! 해제된 메모리 접근
```

Rust에서는 이런 코드가 컴파일조차 되지 않습니다.

### 2. Double-Free (이중 해제)

```c
// C에서 발생하는 버그
int* ptr = malloc(sizeof(int));
free(ptr);  // 첫 번째 해제
free(ptr);  // 두 번째 해제 — 정의되지 않은 동작!
```

Rust의 소유권은 각 값이 정확히 한 번 해제됨을 보장합니다.

### 3. Dangling Pointer (댕글링 포인터)

```c
// C에서 발생하는 버그
int* get_value() {
    int x = 42;
    return &x;  // 스택에 있는 지역 변수의 주소 반환
}   // x는 여기서 사라짐!

int* ptr = get_value();
printf("%d\n", *ptr);  // 버그! 이미 사라진 메모리 접근
```

Rust의 수명(lifetime) 시스템이 이를 방지합니다.

### 4. Memory Leak (메모리 누수)

```c
// C에서 발생하는 버그
void process() {
    int* data = malloc(1024);
    if (!data) {
        return;
    }
    if (should_stop_early()) {
        return;  // free(data)를 호출 못 함 → 누수!
    }
    free(data);
}
```

Rust에서는 값이 스코프를 벗어날 때 자동으로 `drop`이 호출됩니다.

## 소유권의 핵심 아이디어

소유권은 하나의 단순한 아이디어에서 출발합니다:

> **모든 값은 정확히 하나의 소유자(owner)가 있다.**
> **소유자가 스코프를 벗어나면, 값은 자동으로 해제된다.**

이게 전부입니다. 이 단순한 규칙에서 Rust의 모든 메모리 안전성이 나옵니다.

### 스코프와 자동 해제

```rust
fn main() {
    // s는 이 시점에서 아직 존재하지 않음
    {
        let s = String::from("hello");  // s 생성, 힙에 메모리 할당
        println!("{}", s);              // s 사용
    }   // 이 중괄호에서 s의 스코프 끝 → drop(s) 자동 호출 → 메모리 해제

    // println!("{}", s);  // 에러! s는 여기서 존재하지 않음
}
```

TypeScript에서 변수는 스코프를 벗어나도 GC가 나중에 처리합니다:

```typescript
function main() {
    {
        let s = "hello";  // 힙에 문자열 생성
        console.log(s);
    }   // s는 여기서 참조 불가능하지만, GC가 나중에 해제
    // 언제 해제될지 모름
}
```

Rust에서는 `}`를 만나는 순간 즉시, 결정론적으로 해제됩니다.

## 이 장의 구성

소유권 챕터는 세 부분으로 나뉩니다:

1. **소유권 규칙** (2.1): Move vs Copy, String vs &str
2. **참조와 빌림** (2.2): &T, &mut T, 빌림 규칙
3. **슬라이스** (2.3): 문자열 슬라이스, 배열 슬라이스

이 세 개념을 이해하면 Rust의 메모리 모델이 완성됩니다.

## 요약

- 메모리 관리의 세 방법: 수동(C), GC(Java/JS), 소유권(Rust)
- 소유권은 GC 없이 메모리 안전성을 컴파일 타임에 보장
- 블록체인에서 중요: 배포 후 수정 불가, 예측 가능한 성능 필요
- 핵심 아이디어: 모든 값에 정확히 하나의 소유자, 스코프 종료 시 자동 해제

다음 챕터에서 세 가지 소유권 규칙을 구체적인 코드로 배웁니다.
