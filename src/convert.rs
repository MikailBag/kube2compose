/// This module goes from Kubernetes object specs to docker-compose config
use crate::load::ObjectKind;
use anyhow::Context as _;
use linked_hash_map::LinkedHashMap;

pub struct Converter<'a> {
    out: &'a mut crate::compose::Compose,
    objects: &'a [crate::load::Object],
}

impl<'a> Converter<'a> {
    pub fn new(out: &'a mut crate::compose::Compose, objects: &'a [crate::load::Object]) -> Self {
        Self { out, objects }
    }

    fn process_pod_spec(
        &mut self,
        spec: &k8s_openapi::api::core::v1::PodSpec,
        workload_name: &str,
    ) -> anyhow::Result<()> {
        let multiple_containers = spec.containers.len() > 1;
        for container in &spec.containers {
            let svc_name = if multiple_containers {
                format!("{}-{}", workload_name, container.name)
            } else {
                workload_name.to_string()
            };
            let mut svc = crate::compose::Service {
                image: container
                    .image
                    .clone()
                    .context("missing image in container spec")?,
                ports: vec![],
                depends_on: vec![],
                entrypoint: container.command.clone(),
                command: container.args.clone(),
                environment: LinkedHashMap::new(),
            };
            if let Some(env) = &container.env {
                for env_var in env {
                    if let Some(value) = &env_var.value {
                        svc.environment.insert(env_var.name.clone(), value.clone());
                    }
                }
            }
            self.out.services.insert(svc_name, svc);
        }
        Ok(())
    }

    pub fn convert(mut self) -> anyhow::Result<()> {
        for object in self.objects {
            let pod_template_spec = match &object.kind {
                ObjectKind::Deployment(deploy) => &deploy.template,
                ObjectKind::Job(job) => &job.template,
            };
            let pod_spec = pod_template_spec
                .spec
                .as_ref()
                .with_context(|| format!("missing pod spec in object {}", object.name))?;
            self.process_pod_spec(pod_spec, &object.name)?;
        }
        Ok(())
    }
}
