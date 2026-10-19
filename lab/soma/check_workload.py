"""Independent, admitted-code oracle for Soma artifacts, outside workload source."""
import json
import gzip
import os
from pathlib import Path
EXPECTED = {'magit':7,'comint':11,'jupyter':8,'lsp':6,'org':6,'dired':8,'bridge':30,'bridge-daemon':63}


def check(root):
    for name, expected in EXPECTED.items():
        with gzip.open(root/(name+'.json.gz'),'rb') as stream:
            raw = stream.read((64<<20)+1)
        if len(raw)>64<<20:
            raise ValueError('Soma report exceeds bounded decoded size')
        report = json.loads(raw)
        if (report.get('total_scenarios') != expected or report.get('scenarios_as_expected') != expected or
                report.get('insula_checks_passed') is not True or report.get('passed',True) is not True):
            raise ValueError('Soma independent scenario/closure check failed: '+name)


if __name__ == '__main__':
    check(Path(os.environ['HARP_WORKLOAD_OUTPUT']))
