resource "aws___T__instance" "example" {
  ami           = "ami-__T__0c55b159cbfafe1f0"
  instance_type = "t2.micro"
}

resource "aws_instance__T__" "example" {
  instance_type = "t2.__T__micro"
  ami           = "ami-abc123__T__"

  lifecycle {
    precondition {
      condition     = data.aw__T__s_ami.__T__example.architecture__T__ == "x86_64"
      error_message = "The selected AMI must be for the x86_64 architecture."
    }
  }
}

locals {
  service__T___name = "nothing"
  owner             = "Some Team"
  ip                = module.__T__web_server.instance_ip_addr__T__
}

variable "test__T__" {
  type    = string
  default = local.service__T___name
}

data "aw__T__s_ami" "__T__example" {
  most_recent = true

  filter {
    name   = "name"
    values = ["ubuntu/images/hvm-ssd/ubuntu-bionic-18.04-amd64-server-*"]
  }

  filter {
    name   = "virtualization-type"
    values = ["hvm"]
  }

  owners = ["099720109477"] # Canonical
}

resource "aws_s3___T__bucket" "example_bucket" {
  bucket = "my-tf-test-bucket"
}

resource "__T__aws_instance" "example_instance" {
  ami               = "ami-0c55b159cbfafe1f0"
  instance_type     = "t2.micro"
  availability_zone = var.test__T__ == "us-west-1" ? "us-west-1a" : "us-east-1a"

  user_data = <<-EOF
              #!/bin/bash
              echo "BUCKET_NAME=${aws_s3___T__bucket.example_bucket.bucket}" > /etc/environment
              EOF

  tags = {
    Name = "ExampleInstance"
  }
}
