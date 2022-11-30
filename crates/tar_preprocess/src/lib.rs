use bytemuck::NoUninit;
use wgpu::util::DeviceExt;


/// TODO:
/// - automatic uniforms check
/// - automatic structs check
/// - derive macros kinda
/// - ifdef ifndef else endif
/// - automatic bind group layouts check
/// 
pub fn run() {
    // uniforms!(
    //     #[data_name(TestData)]
    //     struct Test {
    //         #[stage(wgpu::ShaderStages::VERTEX)]
    //         test: [u32; 4],
    //     }
    // );
}

pub fn process_defines(source: &mut String, defines: &[String]) {
    let lines = source.lines();
    let mut fin: Vec<String> = vec![];

    let mut add = true;

    

    for line in lines {
        let line = line.trim();

        if line.starts_with("#else") {
            add = !add;
            continue;
        }
        else if line.starts_with("#endif") {
            add = true;
            continue;
        }

        if line.starts_with("#ifdef") {
            let def = defines.contains(&parse_def(line).into());

            add = def;
            continue;
        }
        else if line.starts_with("#ifndef") {
            let def = !defines.contains(&parse_def(line).into());

            add = def;
            continue;
        }

        if add {
            fin.push(line.to_owned() + if line == "\n" || line == "" {""} else {"\n"});
        }
    }

    *source = fin.concat();
}

fn parse_def(line: &str) -> &str {
    line.split(' ').nth(1).unwrap()
}

pub trait Uniforms {
    fn gen_bind_group(&self, layout: &wgpu::BindGroupLayout, device: &wgpu::Device) -> wgpu::BindGroup;
    fn gen_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout;
    fn gen_shader_code(bind_group: u32) -> String;
    fn run_through(input: &mut String, bind_group: u32);
}


pub struct Uniform<T: NoUninit> {
    pub buff: wgpu::Buffer,
    data: T,
}

impl<T: NoUninit> Uniform<T> {
    pub fn new(data: T, usage: String, device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let buff = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some((usage+"buffer").as_str()),
            contents: bytemuck::cast_slice(&[data]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let mut uni = Self { buff, data };

        uni.write_buffer(queue);
        uni
    }

    pub fn update(&mut self, data: T, queue: &wgpu::Queue) {
        self.data = data;
        self.write_buffer(queue);
    }

    fn write_buffer(&mut self, queue: &wgpu::Queue) {
        queue.write_buffer(
            &self.buff,
            0, 
            bytemuck::cast_slice(&[self.data]),
        );
    }
}


/// This insanely cursed macro makes it fairly trivial to generate uniforms 
/// which are accesible on both cpu and gpu
/// 
/// ## Example
/// ```
/// uniforms! {
///     #[data_name(ShaderUniformsData)]
///     pub struct ShaderUniforms {
///         color: [f32; 3]
///     }
/// }
/// ```
#[macro_export]
macro_rules! uniforms {
    (
        #[data_name($DataName:ident)]
        $(#[$outer:meta])*
        $vis:vis struct $StructName:ident {
            $(
                #[stage($Stage:expr)]
                $(#[$inner:ident $($args:tt)*])*
                $Field:ident : $FieldType:ty $(,)?
            )*
        }
    ) => {
        use tar_preprocess::Uniform;
        $(#[$outer])*
        $vis struct $StructName {
            $(
                $Field : Uniform<$FieldType>,
            )*
            pub bind_group: Option<wgpu::BindGroup>,
        }

        $vis struct $DataName {
            $(
                pub $Field : $FieldType,
            )*
        }

        impl $StructName {
            pub fn new(data: $DataName, device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
                use tar_preprocess::Uniform;
                let mut s = Self {
                    bind_group: None,
                    $($Field: Uniform::new(data.$Field, stringify!($Field).into(), device, queue),)*
                };

                let layout = Self::gen_bind_group_layout(device);

                s.bind_group = Some(s.gen_bind_group(&layout, device));

                return s;
            }
        }

        use tar_preprocess::Uniforms;

        impl Uniforms for $StructName {
            fn gen_bind_group(&self, layout: &wgpu::BindGroupLayout, device: &wgpu::Device) -> wgpu::BindGroup {

                let ent = vec![$(self.$Field.buff.as_entire_binding(),)*].iter().enumerate().map(|(binding, resource)| {
                    wgpu::BindGroupEntry {
                        binding: binding as u32,
                        resource: resource.clone(),
                    }
                }).collect::<Vec<wgpu::BindGroupEntry>>();

                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout,
                    entries: ent.as_slice(),
                    label: Some("uniforms bind group")
                })
            }

            fn gen_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
                let ent = vec![$($Stage,)*].iter().enumerate().map(|(binding, stage)| {
                    wgpu::BindGroupLayoutEntry {
                        binding: binding as u32,
                        visibility: *stage,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }
                }).collect::<Vec<wgpu::BindGroupLayoutEntry>>();

                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("uniforms bind group layout"),
                    entries: ent.as_slice(),
                })
            }

            fn gen_shader_code(bind_group: u32) -> String {
                let names = vec![$((stringify!($Field), stringify!($FieldType)),)*];
                let mut res = String::from("");
                for (idx, (name, ty)) in names.iter().enumerate() {
                    use tar_preprocess::rtw_ty;
                    res += format!("@group({bind_group}) @binding({idx})\n").as_str();
                    res += format!("var<uniform> {}: {};", name, rtw_ty(ty)).as_str();
                }

                res
            }

            fn run_through(source: &mut String, bind_group: u32) {
                *source = Self::gen_shader_code(bind_group) + source.as_str();
            }
        }
    };
}


pub fn rtw_ty(ty: &str) -> String{
    let ty = ty.replace(' ', "");
    if ty.starts_with('[') {
        let arr = ty.as_bytes();
        if arr[1] == '[' as u8{
            let size = arr[arr.len()-2] as char;
            let (_, ty) = ty.split(';').next().unwrap().split_at(2);
            return format!("mat{size}x{size}<{ty}>");
        }

        let parts: Vec<&str> = ty.split(';').collect();
        if parts.len() != 2 {
            return "".into();
        }

        let (_, first) = parts[0].split_at(1);
        let (second, _) = parts[1].split_at(parts[1].len()-1);
        

        format!("vec{second}<{first}>")
    }
    else {
        ty
    }
}

macro_rules! __bind_group_entry {
    ($Binding:expr, $Name:ident, $($Other:tt)*) => {
        wgpu::BindGroupEntry {
            binding: $Binding,
            resource: $Name.buff.as_entire_binding(),
        }
        __bind_group_entry(
            $Binding, $($Other)*
        )
    };
}