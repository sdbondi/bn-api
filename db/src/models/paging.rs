use models::SortingDir;

#[derive(Serialize, Deserialize, Clone)]
///struct used to indicate paging information and search query information
pub struct Paging {
    pub page: u32,
    pub limit: u32,
    pub sort: String,
    pub dir: SortingDir,
    pub total: u32,
    pub tags: Vec<SearchParam>,
}

#[derive(Serialize, Deserialize, Clone)]
///Struct used to store search object names and values
pub struct SearchParam {
    pub name: String,
    pub values: Vec<String>,
}

impl Paging {
    pub fn new(page: u32, limit: u32) -> Paging {
        Paging {
            page,
            limit,
            sort: "".to_string(),
            dir: SortingDir::Asc,
            total: 0,
            tags: Vec::new(),
        }
    }
}

impl From<PagingParameters> for Paging {
    fn from(received: PagingParameters) -> Self {
        let default_page = if let Some(i) = received.page { i } else { 0 };
        let default_limit = if let Some(i) = received.limit { i } else { 100 };
        let default_sort = if let Some(ref i) = received.sort {
            i.clone()
        } else {
            "".to_string()
        };
        let default_dir = if let Some(i) = received.dir {
            i
        } else {
            SortingDir::Asc
        };
        let default_tags = if let Some(ref i) = received.tags {
            i.clone()
        } else {
            Vec::new()
        };
        Paging {
            page: default_page,
            limit: default_limit,
            sort: default_sort,
            dir: default_dir,
            total: 0,
            tags: default_tags,
        }
    }
}

#[derive(Serialize, Deserialize)]
///return wrapper struct for returning large lists
pub struct Payload<T> {
    pub data: Vec<T>,
    pub paging: Paging,
}

impl<T> Payload<T> {
    pub fn new(data: Vec<T>, paging_query: Paging) -> Payload<T> {
        let mut payload = Payload {
            data,
            paging: paging_query,
        };
        let total = payload.data.len() as u32;
        payload.paging.total = total;

        payload.paging.limit = total;
        payload
    }

    pub fn empty(paging: Paging) -> Payload<T> {
        let mut payload = Payload {
            data: vec![],
            paging,
        };
        payload.paging.total = 0;
        payload
    }
}

#[derive(Serialize, Deserialize, Clone)]
///struct used to indicate paging information and search query information
pub struct PagingParameters {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub sort: Option<String>,
    pub dir: Option<SortingDir>,
    pub tags: Option<Vec<SearchParam>>,
}