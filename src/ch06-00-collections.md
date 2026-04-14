# 6장: 컬렉션, 클로저, 이터레이터

## Rust 컬렉션 개요

Rust의 표준 라이브러리는 여러 컬렉션 타입을 제공합니다. Node.js의 Array, Map, Set과 유사하지만 메모리 소유권과 함께 동작합니다.

### 주요 컬렉션 타입

| Rust | JavaScript/TypeScript | 설명 |
|------|----------------------|------|
| `Vec<T>` | `Array` | 동적 크기 배열 |
| `String` | `string` | UTF-8 문자열 |
| `HashMap<K, V>` | `Map` | 키-값 저장소 |
| `HashSet<T>` | `Set` | 중복 없는 집합 |
| `BTreeMap<K, V>` | `Map` (정렬됨) | 정렬된 키-값 저장소 |
| `VecDeque<T>` | - | 앞뒤로 삽입/삭제 가능한 큐 |
| `LinkedList<T>` | - | 이중 연결 리스트 |
| `BinaryHeap<T>` | - | 우선순위 큐 |

모든 컬렉션은 **힙에 할당**됩니다. 따라서 소유권 이동, 참조, 클론 개념이 모두 적용됩니다.

### 컬렉션과 소유권

```rust,ignore
fn main() {
    let v = vec![1, 2, 3];

    // 소유권 이동 — v는 더 이상 사용 불가
    let v2 = v;

    // 참조로 접근 — 소유권 이동 없음
    for item in &v2 {
        println!("{}", item);
    }
    println!("Length: {}", v2.len());  // v2는 여전히 유효
}
```

## 이 장의 구성

1. **공통 컬렉션** (6.1): Vec, String, HashMap 심화
2. **클로저** (6.2): Fn, FnMut, FnOnce
3. **이터레이터** (6.3): map, filter, collect 체인

## 블록체인에서의 컬렉션

```rust,ignore
struct Blockchain {
    // Vec: 순서가 있는 블록 목록
    blocks: Vec<Block>,

    // HashMap: 트랜잭션 ID → 트랜잭션 빠른 조회
    tx_index: HashMap<String, Transaction>,

    // HashSet: 사용된 UTXO 추적 (이중 지출 방지)
    spent_outputs: HashSet<String>,
}
```

다음 챕터에서 각 컬렉션을 자세히 배웁니다.
