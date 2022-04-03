# Transaction Processor

Simple transaction processor written as a conversation piece for a job interview.

## Usage

```
transaction-processor [--verbose] <CSV_PATH>
```

### Example run

```
$ transaction-processor tests/test-cases/official.input.txt 
client,available,held,total,locked
2,2.0,0.0,2.0,false
1,1.5,0.0,1.5,false
```

## Notes

* instead of limiting number of decimal places to 4 in the input dataset, the input values are rounded to 4 decimal places 
* maximum amount of money handled per account per item by the processor is limited to `10_000_000_000.0000` moneyes (it may result in pathological cases with MAX money available and MAX money held, but not being able to resolve dispute because MAX + MAX > MAX)
* transactions can be disputed multiple times provided they have been resolved in the meantime
* most anyhow errors should be converted to `thiserror` so that failed transaction can be handled
* output is fixed to four decimal places, but the comparison in integration tests isn't, so just let's not test that
* no care has been taken to make it run fast
* the integration test is a bit hairy, because I run out of time
* written in about 9 hours, which is well above par (2-3 hours in the assignment, but was told 4 hours during interview)
* try googling `total held available withdrawal chargeback "rust" site:github.com` for more projects like this
