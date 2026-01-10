use super::*;

#[test]
fn test_page_request_default() {
    let request = PageRequest::default();
    assert_eq!(request.page, 1);
    assert_eq!(request.per_page, 20);
}

#[test]
fn test_page_request_offset() {
    let request = PageRequest {
        page: 1,
        per_page: 20,
    };
    assert_eq!(request.offset(), 0);

    let request = PageRequest {
        page: 2,
        per_page: 20,
    };
    assert_eq!(request.offset(), 20);
}

#[test]
fn test_page_request_limit() {
    let request = PageRequest {
        page: 1,
        per_page: 50,
    };
    assert_eq!(request.limit(), 50);
}

#[test]
fn test_page_response_new() {
    let data = vec![1, 2, 3];
    let response = PageResponse::new(data.clone(), 1, 10, 3);

    assert_eq!(response.data, data);
    assert_eq!(response.meta.page, 1);
    assert_eq!(response.meta.per_page, 10);
    assert_eq!(response.meta.total, 3);
    assert_eq!(response.meta.total_pages, 1);
}

#[test]
fn test_page_response_pagination() {
    // 25 items, 10 per page -> 3 pages
    let response: PageResponse<i32> = PageResponse::new(vec![], 1, 10, 25);
    assert_eq!(response.meta.total_pages, 3);
}

#[test]
fn test_page_response_empty() {
    let response: PageResponse<i32> = PageResponse::new(vec![], 1, 10, 0);
    assert_eq!(response.meta.total_pages, 1);
}
