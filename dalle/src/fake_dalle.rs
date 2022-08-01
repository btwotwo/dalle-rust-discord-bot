
#[cfg(feature = "fake_dalle")]
pub mod fake_dalle {
    use crate::DalleGenerator;
    use crate::{DalleGenerator, DalleResponse};

    pub struct FakeDalle;

    #[async_trait::async_trait]
    impl DalleGenerator for FakeDalle {
        async fn generate(&self, prompt: &str) -> anyhow::Result<Vec<DalleResponse>> {
            Ok(vec![
                DalleResponse {
                    image_url: "https://cdn.discordapp.com/attachments/857693475633758228/1003608831902892082/f36a7f34-8b60-44f2-90d6-93468f40c23e.webp".into()
                },
                DalleResponse {
                    image_url: "https://cdn.discordapp.com/attachments/857693475633758228/1003608831902892082/f36a7f34-8b60-44f2-90d6-93468f40c23e.webp".into(),
                },
                DalleResponse {
                    image_url: "https://cdn.discordapp.com/attachments/857693475633758228/1003608831902892082/f36a7f34-8b60-44f2-90d6-93468f40c23e.webp".into(),
                }
            ])
        }

        async fn get_task(&self, task_id: &str) -> anyhow::Result<Option<Vec<DalleResponse>>> {
            
        }
    }
}