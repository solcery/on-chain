use {
    borsh::{BorshDeserialize, BorshSchema, BorshSerialize},
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        msg,
        program_error::ProgramError,
        pubkey::Pubkey,
    },
};

#[derive(BorshSerialize, BorshDeserialize, BorshSchema, Debug, PartialEq)]
pub enum SolceryType {
    Error,
    SBool,
    SInt,
    SString,
    SUrl,
    SLink { template: Pubkey },
    SBrick { brick_type: u32 }, //TODO
    SArray { nested_type: SolceryNestedType },
}

#[derive(BorshSerialize, BorshDeserialize, BorshSchema, Debug, PartialEq)]
pub enum SolceryNestedType {
    Error,
    SBool,
    SInt,
    SString,
    SUrl,
    SLink { template: Pubkey },
}

#[derive(Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct TemplateData {
    pub id: u32,
    pub name: String,
    pub code: String,
    pub storages: Vec<Pubkey>,
    pub max_field_index: u32,
    pub fields: Vec<Field>,
    pub custom_data: Vec<u8>,
}

#[derive(Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct Field {
    pub id: u32,
    pub params: FieldParams,
}

#[derive(Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct FieldParams {
    pub field_type: SolceryType,
    pub name: String,
    pub code: String,
    pub construct_client: bool,
    pub construct_server: bool,
}

pub fn process_instruction(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    let (tag, data) = instruction_data.split_first().unwrap();
    let accounts_iter = &mut accounts.iter();
    match tag {
        0 => {
            let project_info = next_account_info(accounts_iter)?;
            let template_info = next_account_info(accounts_iter)?;
            let storage_info = next_account_info(accounts_iter)?;
            let project_templates_storage_info = next_account_info(accounts_iter)?;
            create(
                project_info,
                template_info,
                storage_info,
                project_templates_storage_info,
            )
        }
        1 => {
            let template_info = next_account_info(accounts_iter)?;
            let field_params = FieldParams::deserialize(&mut &*data)?;
            add_field(template_info, field_params)
        }
        2 => {
            let template_info = next_account_info(accounts_iter)?;
            msg!("{:?}", data);
            let field_id = u32::deserialize(&mut &*data)?;
            delete_field(template_info, field_id)
        }
        3 => {
            let template_info = next_account_info(accounts_iter)?;
            let name = String::deserialize(&mut &*data)?;
            change_name(template_info, name)
        }
        4 => {
            let template_info = next_account_info(accounts_iter)?;
            let code = String::deserialize(&mut &*data)?;
            change_code(template_info, code)
        }
        _ => Err(ProgramError::InvalidAccountData),
    }
}

pub fn create(
    project_info: &AccountInfo,
    template_info: &AccountInfo,
    storage_info: &AccountInfo,
    project_templates_storage_info: &AccountInfo,
) -> ProgramResult {
    msg!("Template/Create");
    let new_template_data = TemplateData {
        id: solcery_project::get_uniq_id(project_info),
        name: "New template".to_string(),
        code: "newTemplate".to_string(),
        storages: vec![*storage_info.key], // TODO: template without storage
        max_field_index: 10,
        fields: vec![Field {
            id: 1,
            params: FieldParams {
                field_type: SolceryType::SString,
                name: String::from("name"),
                code: String::from("name"),
                construct_client: true,
                construct_server: false,
            },
        }],
        custom_data: Vec::new(),
    };
    solcery_crud::initialize(project_info, template_info);
    solcery_crud::write(template_info, 0, new_template_data.try_to_vec().unwrap());
    solcery_storage::assign(project_info, storage_info, template_info)?;
    solcery_storage::add(project_templates_storage_info, template_info)?;
    Ok(())
}

pub fn add_field(template_info: &AccountInfo, field_params: FieldParams) -> ProgramResult {
    msg!("Template/AddField");
    let mut template = {
        let template_data =
            &template_info.data.borrow()[solcery_crud::RecordData::WRITABLE_START_INDEX..];
        TemplateData::deserialize(&mut &*template_data)?
    };
    template.max_field_index += 1;
    let field = Field {
        id: template.max_field_index,
        params: field_params,
    };
    template.fields.push(field);
    solcery_crud::write(template_info, 0, template.try_to_vec().unwrap());
    Ok(())
}

pub fn delete_field(template_info: &AccountInfo, field_id: u32) -> ProgramResult {
    msg!("Template/DeleteField");
    let mut template = {
        let template_data =
            &template_info.data.borrow()[solcery_crud::RecordData::WRITABLE_START_INDEX..];
        TemplateData::deserialize(&mut &*template_data)?
    };
    let index_of_field_id = template.fields.iter().position(|x| x.id == field_id);
    match index_of_field_id {
        Some(ind) => {
            template.fields.remove(ind);
            solcery_crud::write(template_info, 0, template.try_to_vec().unwrap());
            Ok(())
        }
        _ => Err(ProgramError::InvalidAccountData),
    }
}

pub fn change_name(template_info: &AccountInfo, name: String) -> ProgramResult {
    let mut template = {
        let template_data =
            &template_info.data.borrow()[solcery_crud::RecordData::WRITABLE_START_INDEX..];
        TemplateData::deserialize(&mut &*template_data)?
    };
    template.name = name;
    solcery_crud::write(template_info, 0, template.try_to_vec().unwrap());
    Ok(())
}

pub fn change_code(template_info: &AccountInfo, code: String) -> ProgramResult {
    let mut template = {
        let template_data =
            &template_info.data.borrow()[solcery_crud::RecordData::WRITABLE_START_INDEX..];
        TemplateData::deserialize(&mut &*template_data)?
    };
    template.code = code;
    msg!("{:?}", template);
    solcery_crud::write(template_info, 0, template.try_to_vec().unwrap());
    Ok(())
}
