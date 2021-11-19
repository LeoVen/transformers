import torch
from tokenizers import ByteLevelBPETokenizer
from transformers import GPT2Tokenizer, GPT2LMHeadModel

model_folder = './model/GPT2-7'

BOS_TOKEN = '<|startoftext|>'
EOS_TOKEN = '<|endoftext|>'
PAD_TOKEN = '<|pad|>'

tokenizer = GPT2Tokenizer.from_pretrained(
    'gpt2',
    bos_token = BOS_TOKEN,
    eos_token = EOS_TOKEN,
    pad_token = PAD_TOKEN,
)

def decode(tokenizer, sample_output):
    return tokenizer.decode(sample_output, skip_special_tokens=True).replace("<N><N>", "\n").replace("<N>", "\n")

model = GPT2LMHeadModel.from_pretrained(model_folder, local_files_only=True).cuda()

while True:
    sent = input("sent:")

    if sent == 'q':
        break

    max_len = int(input("len:"))

    generated = tokenizer("<|startoftext|>" + sent, return_tensors="pt").input_ids.cuda()

    samples = model.generate(generated, do_sample=True, num_return_sequences=5, max_length=max_len)

    for sample in samples:
        print(len(sample), end=":\n")
        print(decode(tokenizer, sample), end="\n+--------------------------------------------------------------------------------+\n")

    print('\n\n\n')
