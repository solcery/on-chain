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

// impl From<UTF8Error> for ProgramError {
//     fn from(e: UTF8Error) -> Self {
//         ProgramError::Custom(e as u32)
//     }
// }

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct Project {
    pub name: String,
    pub owner: Pubkey,
    pub template_storage: Pubkey,
    pub uniq_id: u32,
}

#[must_use]
pub fn check_access(user_info: &AccountInfo, project_info: &AccountInfo) -> bool {
    let record_data =
        solcery_crud::RecordData::deserialize(&mut &project_info.data.borrow()[..]).unwrap();
    let project = Project::deserialize(&mut &record_data.data.bytes[..]).unwrap();
    project.owner == *user_info.key
}

pub fn process_instruction(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    let (tag, _data) = instruction_data.split_first().unwrap();
    let accounts_iter = &mut accounts.iter();
    match tag {
        0 => {
            let project_info = next_account_info(accounts_iter)?;
            let project_templates_storage_info = next_account_info(accounts_iter)?;
            let owner_info = next_account_info(accounts_iter)?;
            create(project_info, project_templates_storage_info, owner_info)
        }
        _ => Err(ProgramError::InvalidAccountData),
    }
}

pub fn create(
    project_info: &AccountInfo,
    project_templates_storage_info: &AccountInfo,
    owner_info: &AccountInfo,
) -> ProgramResult {
    msg!("Project/Create");
    let project_data = Project {
        name: "New project".to_string(), // TODO: name
        owner: *owner_info.key,
        template_storage: *project_templates_storage_info.key,
        uniq_id: 0,
    };
    solcery_crud::initialize(project_info, project_info);
    solcery_crud::write(project_info, 0, project_data.try_to_vec().unwrap());
    solcery_storage::assign(project_info, project_templates_storage_info, project_info)?;
    Ok(())
}

#[must_use]
pub fn get_uniq_id(project_info: &AccountInfo) -> u32 {
    let mut project_data = Project::deserialize(
        &mut &project_info.data.borrow()[solcery_crud::RecordData::WRITABLE_START_INDEX..],
    )
    .unwrap();
    project_data.uniq_id += 1;
    solcery_crud::write(project_info, 0, project_data.try_to_vec().unwrap());
    project_data.uniq_id - 1
}
